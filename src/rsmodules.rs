/*
MIT License

Copyright (c) 2017 Frederik Delaere

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use super::bold;
use super::output;
use glob::glob_with;
use glob::MatchOptions;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::Ordering;

use mdcat;

use mdcat::{ResourceAccess, TerminalCapabilities, TerminalSize};
use pulldown_cmark::Parser;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::stdin;
use syntect::parsing::SyntaxSet;

mod autoload;
mod cache;
mod manage;
mod script;

static DEFAULT_MODULE_PATH: &str = "/usr/local";
static ENV_LOADEDMODULES: &str = "LOADEDMODULES"; // name of an env var
static ENV_UNDO: &str = "RSMODULES_UNDO"; // name of an env var

#[derive(Debug)]
pub struct Rsmodule<'a> {
    pub cmd: &'a str,                 // load|list|avail|...
    pub typed_command: &'a str,       // load|list|avail|...
    pub arg: &'a str,                 // blast/12.1 | blast | blast/12
    pub search_path: &'a Vec<String>, // module paths
    pub shell: &'a str,               // tcsh|csh|bash|zsh
    pub shell_width: usize,
}

pub fn crash(signal: i32, message: &str) {
    let tmp_file_path = super::TMPFILE_PATH.lock().unwrap();

    if super::TMPFILE_INITIALIZED.load(Ordering::Relaxed) {
        let path = &(*tmp_file_path);
        fs::remove_file(path).unwrap();
    }

    crash!(signal, "{}", message);
}

pub fn get_module_paths(silent: bool) -> Vec<String> {
    let mut modulepath: String = String::from(DEFAULT_MODULE_PATH);
    let mut modulepaths: Vec<String> = Vec::new();

    match env::var("MODULEPATH") {
        Ok(path) => modulepath = path,
        Err(_) => {
            if !silent {
                show_warning!("$MODULEPATH not found, using {}", modulepath);
            }
            return modulepaths;
        }
    };

    if modulepath.is_empty() {
        return modulepaths;
    }

    let modulepath: Vec<&str> = modulepath.split(':').collect();
    for path in modulepath {
        modulepaths.push(path.to_string());
    }

    modulepaths
}

pub fn get_module_list(shell: &str) -> Vec<(String, i64)> {
    let mut modules: Vec<(String, i64)> = Vec::new();
    let mut found_cachefile: bool = false;
    let modulepaths = get_module_paths(false);

    for path in modulepaths {
        // test if cachefiles exist in the paths
        // if they don't and we have write permission in that folder
        // we should create the cache
        let mut testpath = PathBuf::from(&path);
        testpath.push(cache::MODULESINDEX);

        if testpath.exists() {
            cache::parse_modules_cache_file(&testpath, &mut modules);
            found_cachefile = true;
        } else {
            echo(
                &format!("  {}: {} doesn't contain an index.", bold(shell, "WARNING"), path),
                shell,
            );
            if cache::update(&path, shell) {
                cache::parse_modules_cache_file(&testpath, &mut modules);
                found_cachefile = true;
            }
        }
    }

    if !found_cachefile {
        crash(super::CRASH_NO_CACHE_FILES_FOUND, "No cachefiles found.");
    }

    modules.sort();
    modules
}

pub fn get_shell_info() -> (String, usize) {
    // the shell argument can either be 'bash', 'tcsh'
    // or the shellname comma shellwidth
    // bash,80 or csh,210 or bash,210 etc
    // if no width is specified, 80 is used as default widtho
    let args: Vec<String> = env::args().collect();
    let err_return = (String::from("noshell"), 80);

    if args.len() == 1 {
        return err_return;
    }

    if args.len() >= 2 && (&args[1] == "-h" || &args[1] == "--help") {
        crash(super::CRASH_GET_SHELL, "Cannot get shell.");
        return err_return;
    }

    let mut shell: &str = &args[1];
    let mut shell_width: usize = 80;

    let shell_split: Vec<&str> = shell.split(',').collect();

    if shell_split.len() == 2 {
        if shell_split[1] != "" {
            shell_width = match FromStr::from_str(shell_split[1]) {
                Ok(w) => w,
                Err(_) => 80,
            };
        }
        shell = shell_split[0];
    }

    (shell.to_string(), shell_width)
}

pub fn command(rsmod: &mut Rsmodule) {
    if rsmod.cmd == "load" {
        module_action(rsmod, "load");
    } else if rsmod.cmd == "unload" {
        module_action(rsmod, "unload");
    } else if rsmod.cmd == "switch" {
        let args: Vec<&str> = rsmod.arg.split_whitespace().collect();

        if args.len() < 2 {
            return;
        }
        let unload = args[0];
        let load = args[1];
        if !is_module_loaded(unload, false) {
            return;
        }
        rsmod.arg = unload;
        module_action(rsmod, "unload");
        rsmod.arg = load;
        module_action(rsmod, "load");
    } else if rsmod.cmd == "available" {
        let args: Vec<&str> = rsmod.arg.split_whitespace().collect();
        let mut arg: &str = "";
        let mut only_default: bool = false;
        if args.len() == 2 {
            if args[0] == "--default" {
                only_default = true;
            }
            arg = &args[1];
        } else if args.len() == 1 {
            if args[0] == "--default" {
                only_default = true;
            } else {
                arg = &args[0];
            }
        }
        cache::get_module_list(arg, rsmod.typed_command, rsmod.shell, rsmod.shell_width, only_default);
    } else if rsmod.cmd == "list" {
        list(rsmod);
    } else if rsmod.cmd == "purge" {
        purge(rsmod);
    } else if rsmod.cmd == "refurbish" {
        refurbish(rsmod);
    } else if rsmod.cmd == "refresh" {
        refresh(rsmod);
    } else if rsmod.cmd == "info" {
        module_action(rsmod, "info");
    } else if rsmod.cmd == "makecache" {
        let modulepaths = get_module_paths(false);
        for modulepath in modulepaths {
            if modulepath != "" {
                cache::update(&modulepath, rsmod.shell);
            }
        }
    } else if rsmod.cmd == "undo" {
        undo(rsmod);
    } else if rsmod.cmd == "delete" {
        manage::delete(rsmod);
    } else if rsmod.cmd == "create" {
        manage::create(rsmod);
    } else if rsmod.cmd == "autoload" {
        autoload(rsmod);
    } else if rsmod.cmd == "readme" {
        module_action(rsmod, "readme");
    }
}

pub fn get_module_description(path: &PathBuf, action: &str) -> Vec<String> {
    script::run(path, action);

    script::get_description()
}

fn find_root(path: &str, previous_path: &str, selected_module: &str) -> String {
    let parts: Vec<&str> = selected_module.split('/').collect();

    let mut counter: usize = 0;

    //for part in parts.iter() {
    for part in &parts {
        if path.find(part) != None {
            counter += 1;
        }
    }

    if counter == parts.len() {
        let mut parent = PathBuf::from(&path);
        parent.pop();
        find_root(parent.to_str().unwrap(), path, selected_module)
    } else {
        String::from(previous_path)
    }

    //String::from(previous_path)
}

fn glob_path(glob: &str, paths: &[String]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    for path in paths.iter() {
        for entry in glob_with(&format!("{}/{}", path, glob), &options).expect("Failed to read man glob pattern") {
            match entry {
                Ok(path) => result.push(String::from(path.to_str().unwrap())),
                Err(_e) => {}
            }
        }
    }

    result
}

fn read_input<T: AsRef<str>>(filename: T) -> std::io::Result<(PathBuf, String)> {
    let cd = std::env::current_dir()?;
    let mut buffer = String::new();

    if filename.as_ref() == "-" {
        stdin().read_to_string(&mut buffer)?;
        Ok((cd, buffer))
    } else {
        let mut source = File::open(filename.as_ref())?;
        source.read_to_string(&mut buffer)?;
        let base_dir = cd.join(filename.as_ref()).parent().map(|p| p.to_path_buf()).unwrap_or(cd);
        Ok((base_dir, buffer))
    }
}

fn get_readme(selected_module: &str, shell: &str) -> Vec<String> {
    // first check if there are manfiles
    let manpaths = script::get_readme_manpaths();

    let parts: Vec<&str> = selected_module.split('/').collect();

    let name = if !parts.is_empty() { parts[0] } else { "" };

    let mut mans: Vec<String> = Vec::new();

    // search filesystem with a glob
    mans.extend_from_slice(&glob_path(&format!("man1/{}.1", name), &manpaths));
    mans.extend_from_slice(&glob_path(&format!("man1/{}.1", name.to_lowercase()), &manpaths));
    mans.extend_from_slice(&glob_path(&format!("man1/{}.1", name.to_uppercase()), &manpaths));
    mans.extend_from_slice(&glob_path(&format!("/{}.1", name), &manpaths));
    mans.extend_from_slice(&glob_path(&format!("/{}.1", name.to_lowercase()), &manpaths));
    mans.extend_from_slice(&glob_path(&format!("/{}.1", name.to_uppercase()), &manpaths));

    if !mans.is_empty() {
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("man -P cat {}", mans[0]));

        return lines;
    }

    // no manpages
    // so check for readmes

    let paths = script::get_readme_paths();

    let mut readme_paths: Vec<String> = Vec::new();

    for path in &paths {
        let mut tmp = find_root(path, path, selected_module);
        if !tmp.ends_with('/') {
            tmp.push('/');
        }
        readme_paths.push(tmp);
    }

    //readme_paths.sort();
    readme_paths.sort_by(|a, b| a.len().cmp(&b.len()));
    readme_paths.dedup();

    let mut readmes: Vec<String> = Vec::new();

    // search filesystem with a glob
    readmes.extend_from_slice(&glob_path("*read*me*", &readme_paths));
    readmes.extend_from_slice(&glob_path("doc*/*read*me*", &readme_paths));
    readmes.extend_from_slice(&glob_path("*/doc*/*read*me*", &readme_paths));
    readmes.extend_from_slice(&glob_path("*/src*/*read*me*", &readme_paths));
    readmes.extend_from_slice(&glob_path("src*/*read*me*", &readme_paths));
    readmes.extend_from_slice(&glob_path("*/source*/*read*me*", &readme_paths));
    readmes.extend_from_slice(&glob_path("source*/*read*me*", &readme_paths));

    //readmes.sort();
    readmes.sort_by(|a, b| a.len().cmp(&b.len()));

    let mut lines: Vec<String> = Vec::new();
    let mut counter = 0;
    for readme in &readmes {
        if counter == 0 {
            let mdtmpfile = format!("{}{}", super::TMPFILE_PATH.lock().unwrap(), ".md");
            lines.push(format!("echo '\n  {}{}\n'", bold(shell, "Showing readme file: "), readme));
            let path = Path::new(readme);

            let markdown = path.extension() != None && path.extension().unwrap() == "md";

            match OpenOptions::new().write(true).create(true).truncate(true).open(&mdtmpfile) {
                Ok(fileresult) => {
                    let mut file: File = fileresult;
                    if markdown && (shell == "zsh" || shell == "bash") && env::var("NO_COLOR").is_err() {
                        let (base_dir, input) = read_input(readme).unwrap();
                        let parser = Parser::new(&input);

                        let syntax_set = SyntaxSet::load_defaults_newlines();
                        mdcat::push_tty(
                            &mut file,
                            TerminalCapabilities::detect(),
                            TerminalSize::detect().unwrap_or_default(),
                            parser,
                            &base_dir,
                            ResourceAccess::LocalOnly,
                            syntax_set,
                        )
                        .unwrap();

                        lines.push(format!("echo \"`cat {}`\"", mdtmpfile));
                        lines.push(format!("rm -f {}", mdtmpfile));
                    } else {
                        lines.push(format!("cat {}", readme));
                    }
                }
                Err(_) => lines.push(format!("cat {}", readme)),
            };

            if readmes.len() > 1 {
                lines.push(format!("echo '\n  {}\n'", bold(shell, "Other possible readme files: ")));
            }
        } else {
            lines.push(format!("echo '  - {}'", readme));
        }
        counter += 1;
    }

    if counter == 0 {
        lines.push(format!("echo '\n  {}'", bold(shell, "No readme found.")));

        let str_path = if readme_paths.len() > 1 { "paths" } else { "path" };

        lines.push(format!(
            "echo '\n  This script is not perfect, you can try searching in the following {}:\n'",
            str_path
        ));
        for path in &readme_paths {
            lines.push(format!("echo '  - {}'", path));
        }
    }

    lines.push(String::from("echo ''"));

    lines
}

fn run_modulefile(path: &PathBuf, rsmod: &mut Rsmodule, selected_module: &str, action: &str) {
    script::run(path, action);

    let data = if action == "info" {
        script::get_info(rsmod.shell, selected_module)
    } else if action == "readme" {
        get_readme(selected_module, rsmod.shell)
    } else {
        script::get_output(selected_module, action, rsmod.shell)
    };

    for mut line in data {
        if rsmod.shell != "perl" {
            line = format!("{}\n", line);
        }

        if rsmod.shell == "noshell" || rsmod.shell == "python" || rsmod.shell == "perl" {
            println!("{}", line);
        } else {
            output(line);
        }
    }
}

fn module_action(rsmod: &mut Rsmodule, action: &str) {
    // when unloading we only want a list of the loaded modules
    // for matching modulenames :
    // we have: blast/1.2 and blast/1.3 (D) while blast/1.2 is loaded
    // and blast/1.3 is not loaded
    // module unload blast
    // should unload blast/1.2 and not blast/1.3
    let mut reversed_modules = if action == "unload" {
        get_loaded_list()
    } else {
        get_module_list(rsmod.shell)
    };

    reversed_modules.reverse();

    if rsmod.arg == "" {
        // TODO: only print usage info about this subcommand
        super::usage(true);
        return;
    }

    //let mut selected_module = rsmod.arg;
    let mut modulefile: PathBuf = PathBuf::new();
    let mut found: bool;

    let modules: Vec<&str> = rsmod.arg.split_whitespace().collect();

    for mdl in modules {
        let mut selected_module = mdl;
        found = false;

        // check if module file exists
        // run over modulepaths, check if a folder/file exists with the wanted 'module' var

        // if not, maybe check if its a partial match
        // blast -> blast/x86_64/1.0 and blast/x86_64/2.0
        // then we need to load the Default version
        // or just the latest one

        'outer: for modulepath in rsmod.search_path {
            let testpath = format!("{}/{}", modulepath, mdl);
            if Path::new(&testpath).exists() {
                // we got it, now we need to figure out if its a partial match or not
                if Path::new(&testpath).is_file() {
                    found = true;
                    modulefile = PathBuf::from(&testpath);
                } else {
                    for module in &reversed_modules {
                        // we got a partial match, now we need to find the default module
                        // for this folder or subfolders
                        // loop through all the modules and get the first one
                        // that matches starts_with

                        // partial matches only work for file/folder names
                        // blast or blast/x86_64 but not blas or blast/x86_
                        // because of the above 'exists()' check

                        // prevent that: module load blast loads blastz
                        let splitter: Vec<&str> = module.0.split(mdl).collect();
                        if splitter.len() > 1 {
                            if found && module.0.starts_with(mdl) && module.1 == 1 {
                                selected_module = module.0.as_ref();
                                let testpath = format!("{}/{}", modulepath, module.0);
                                // if a modulefile is already found, don't overwrite it with a
                                // possible non existing file
                                if Path::new(&testpath).exists() && Path::new(&testpath).is_file() {
                                    modulefile = PathBuf::from(&testpath);
                                    break 'outer;
                                }
                            }

                            if found && !module.0.starts_with(mdl) {
                                break 'outer;
                            }

                            if !found && splitter[1].starts_with('/') && module.0.starts_with(mdl) {
                                selected_module = module.0.as_ref();
                                found = true;
                                let testpath = format!("{}/{}", modulepath, module.0);
                                // if a modulefile is already found, don't overwrite it with a
                                // possible non existing file
                                if Path::new(&testpath).exists() && Path::new(&testpath).is_file() {
                                    modulefile = PathBuf::from(&testpath);
                                }
                                // don't break out of the outer loop, their might be a module
                                // file marked as D
                                //break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if !found && action != "unload" {
            crash(
                super::CRASH_MODULE_NOT_FOUND,
                &format!("Module {} not found.", selected_module),
            );
        }
        // check of another version is already loaded
        // and replace it with the current one

        let mut replaced_module: bool = false;
        let mut other: String = String::new();
        if is_other_version_of_module_loaded(selected_module) && action == "load" {
            let parts: Vec<&str> = selected_module.split('/').collect();
            let tmp_selected_module = parts[0];

            other = get_other_version_of_loaded_module(tmp_selected_module);

            if other != "" && other != selected_module {
                for modulepath in rsmod.search_path {
                    let testpath = format!("{}/{}", modulepath, other);
                    if Path::new(&testpath).exists() && Path::new(&testpath).is_file() {
                        let tmpmodulefile: PathBuf = PathBuf::from(&testpath);
                        // unload the module as we found the path to the file
                        run_modulefile(&tmpmodulefile, rsmod, other.as_ref(), "unload");
                        replaced_module = true;
                    }
                }
            }
        }

        // check if we are already loaded (LOADEDMODULES env var)
        if is_module_loaded(selected_module, false) && action == "load" {
            // unload the module
            run_modulefile(&modulefile, rsmod, selected_module, "unload");
            // load the module again
            run_modulefile(&modulefile, rsmod, selected_module, "load");
            continue;
        }

        // don't unload if we are not loaded in the first place
        if !is_module_loaded(selected_module, false) && action == "unload" {
            continue;
        }

        // finaly load|unload|info the module
        output(format!("# {} {}\n", action, selected_module));
        run_modulefile(&modulefile, rsmod, selected_module, action);

        if replaced_module && other != "" && selected_module != "" {
            let spaces = if rsmod.shell == "noshell" || rsmod.shell == "perl" || rsmod.shell == "python" {
                ""
            } else {
                "  "
            };

            let msg: String = format!(
                "{}The previously loaded module {} has been replaced \
                 with {}",
                spaces,
                bold(rsmod.shell, &other),
                bold(rsmod.shell, selected_module)
            );
            if rsmod.shell != "noshell" {
                echo("", rsmod.shell);
            }
            echo(&msg, rsmod.shell);
            if rsmod.shell != "noshell" {
                echo("", rsmod.shell);
            }
        }
    }
}

pub fn is_module_loaded(name: &str, only_full_match: bool) -> bool {
    if name == "" {
        return false;
    }

    let loadedmodules: String;
    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return false;
        }
    };

    let loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    for module in loadedmodules {
        // full match
        if module == name {
            return true;
        }

        // partial match (part before the slash)
        let part_module: Vec<&str> = module.split('/').collect();
        let part_name: Vec<&str> = name.split('/').collect();

        if part_module.is_empty() || part_name.is_empty() {
            continue;
        }

        if part_module[0] == part_name[0] && !only_full_match {
            return true;
        }
    }

    false
}

pub fn get_other_version_of_loaded_module(name: &str) -> String {
    let loadedmodules: String;
    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return String::new();
        }
    };

    let parts: Vec<&str> = name.split('/').collect();
    let part = parts[0];

    let loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    for module in loadedmodules {
        let module_parts: Vec<&str> = module.split('/').collect();
        let module_part = module_parts[0];
        if part == module_part {
            return module.to_string();
        }
    }

    String::new()
}

pub fn is_other_version_of_module_loaded(name: &str) -> bool {
    let loadedmodules: String;
    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return false;
        }
    };

    let parts: Vec<&str> = name.split('/').collect();
    let part = parts[0];

    let loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    for module in loadedmodules {
        let module_parts: Vec<&str> = module.split('/').collect();
        let module_part = module_parts[0];
        if part == module_part {
            return true;
        }
    }

    false
}

pub fn echo(line: &str, shell: &str) {
    //FIXME: if line contains \n and shell is csh or tcsh
    // escape it
    if shell == "noshell" || shell == "progressbar" {
        println!("{}", line);
    } else if shell == "python" {
        println!("print(\"{}\")", line);
    } else if shell == "perl" {
        println!("print(\"{}\\n\");", line);
    } else {
        let data = format!("echo \"{}\"\n", line);
        output(data);
    }
}

pub fn get_loaded_list() -> Vec<(String, i64)> {
    let loadedmodules: String;
    let mut result: Vec<(String, i64)> = Vec::new();

    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return Vec::new();
        }
    };

    for module in loadedmodules.split(':') {
        if module != "" {
            result.push((module.to_string(), 1));
        }
    }
    result.sort();

    result
}

fn list(rsmod: &mut Rsmodule) {
    let loadedmodules: String;

    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return;
        }
    };

    let mut loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    loadedmodules.retain(|&x| x != "");
    // display loaded modules in loaded order
    loadedmodules.reverse();

    if !loadedmodules.is_empty() {
        if rsmod.shell != "noshell" {
            echo("", rsmod.shell);
            echo("  Currently loaded modules:", rsmod.shell);
            echo("", rsmod.shell);
        }
    } else {
        let spaces = if rsmod.shell == "noshell" || rsmod.shell == "perl" || rsmod.shell == "python" {
            ""
        } else {
            "  "
        };

        echo("", rsmod.shell);
        echo(&format!("{}There are no modules loaded.", spaces), rsmod.shell);
    }
    for module in loadedmodules {
        if module != "" {
            if rsmod.shell == "noshell" {
                echo(module, rsmod.shell);
            } else {
                echo(&format!("  * {}", bold(rsmod.shell, module)), rsmod.shell);
            }
        }
    }
    if rsmod.shell != "noshell" {
        echo("", rsmod.shell);
    }
}

fn refresh(rsmod: &mut Rsmodule) {
    let loadedmodules: String;

    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return;
        }
    };

    let loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    for module in loadedmodules {
        if module != "" {
            let mut rsmod_command: Rsmodule = Rsmodule {
                cmd: "load",
                typed_command: "load",
                arg: module,
                search_path: rsmod.search_path,
                shell: rsmod.shell,
                shell_width: rsmod.shell_width,
            };
            command(&mut rsmod_command);
        }
    }
}

fn purge(rsmod: &mut Rsmodule) {
    let loadedmodules: String;

    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return;
        }
    };

    let loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    for module in loadedmodules {
        if module != "" {
            let mut rsmod_command: Rsmodule = Rsmodule {
                cmd: "unload",
                typed_command: "unload",
                arg: module,
                search_path: rsmod.search_path,
                shell: rsmod.shell,
                shell_width: rsmod.shell_width,
            };
            command(&mut rsmod_command);
        }
    }
}

fn refurbish(rsmod: &mut Rsmodule) {
    purge(rsmod);
    let mut args: Vec<&str> = Vec::new(); //rsmod.arg.split_whitespace().collect();
                                          //let mut subcommand = args.remove(0);
                                          //args.remove(0);
    let subcommand = "refurbish";
    autoload::run(subcommand, &mut args, rsmod.shell);
}

fn undo(rsmod: &mut Rsmodule) {
    let args = match env::var(ENV_UNDO) {
        Ok(list) => list,
        Err(_) => {
            return;
        }
    };
    let mut args: Vec<&str> = args.split_whitespace().collect();

    let mut cmd: &str;

    if args.is_empty() {
        return;
    }

    if args.len() > 1 {
        // means we did a purge
        cmd = args[0];
        if cmd == "load" {
            cmd = "unload";
        } else if cmd == "unload" {
            cmd = "load";
        }
        args.retain(|&i| (i != "load" && i != "unload" && i != "switch"));
        let mut rsmod_command: Rsmodule = Rsmodule {
            cmd,
            typed_command: cmd,
            arg: &args.join(" "),
            search_path: rsmod.search_path,
            shell: rsmod.shell,
            shell_width: rsmod.shell_width,
        };
        command(&mut rsmod_command);
        if cmd == "switch" {
            args.reverse();
        }

        output(super::setenv(
            "RSMODULES_UNDO",
            &format!("{} {}", cmd, args.join(" ")),
            rsmod.shell,
        ));
    }
}

fn autoload_usage(shell: &str) {
    echo("", shell);
    echo(
        &format!("  {}: module autoload [subcommand] [modulename(s)]", bold(shell, "Usage")),
        shell,
    );
    echo("", shell);
    echo("  The module autoload command manages which modules that", shell);
    echo("  are autoloaded in your environment.", shell);
    echo("", shell);
    echo("  The following subcommands are available:", shell);
    echo("", shell);
    echo(&format!("{}", bold(shell, "    * append [modulename(s)]")), shell);
    echo(
        "      Adds one or more module to the end of the list of autoloaded modules.",
        shell,
    );
    echo("", shell);
    echo(&format!("{}", bold(shell, "    * prepend [modulename(s)]")), shell);
    echo(
        "      Adds one or more module to the beginning of the list of autoloaded modules.",
        shell,
    );
    echo("", shell);
    echo(&format!("{}", bold(shell, "    * remove [modulename(s)]")), shell);
    echo(
        "      Removes one or more module from the \
         list of autoloaded moules.",
        shell,
    );
    echo("", shell);
    echo(&format!("{}", bold(shell, "    * list")), shell);
    echo("      Shows a list of all autoloaded modules.", shell);
    echo("", shell);
    echo(&format!("{}", bold(shell, "    * purge")), shell);
    echo("      Removes all the autoloaded modules.", shell);
    echo("", shell);
}

fn autoload(rsmod: &mut Rsmodule) {
    let mut args: Vec<&str> = rsmod.arg.split_whitespace().collect();

    if args.is_empty() {
        autoload_usage(rsmod.shell);
        return;
    }

    // TODO: allow only for append, prepend, remove, list
    let subcommand = args.remove(0);

    if subcommand != "append" && subcommand != "prepend" && subcommand != "remove" && subcommand != "list" {
        autoload_usage(rsmod.shell);
        return;
    }

    autoload::run(subcommand, &mut args, rsmod.shell);
}

#[cfg(test)]
mod tests {
    use super::get_other_version_of_loaded_module;
    use super::is_module_loaded;
    use super::is_other_version_of_module_loaded;
    use std::env;

    #[test]
    fn _is_other_version_of_module_loaded() {
        env::set_var("LOADEDMODULES", "blast/12.3:blast/11.1");
        assert_eq!(true, is_other_version_of_module_loaded("blast/11.1"));
        assert_eq!(true, is_other_version_of_module_loaded("blast/13.4"));
        assert_eq!(true, is_other_version_of_module_loaded("blast"));
        assert_eq!(true, is_other_version_of_module_loaded("blast/x86_64/1"));
        assert_eq!(false, is_other_version_of_module_loaded("perl"));
        assert_eq!(false, is_other_version_of_module_loaded(""));
    }
    #[test]
    fn _get_other_version_of_module_loaded() {
        env::set_var("LOADEDMODULES", "blast/12.3:blast/11.1");
        assert_eq!("blast/12.3", get_other_version_of_loaded_module("blast/11.1"));
        assert_eq!("blast/12.3", get_other_version_of_loaded_module("blast/x86_64/11.1"));
        assert_eq!("", get_other_version_of_loaded_module("perl"));
        assert_eq!("", get_other_version_of_loaded_module(""));
    }
    #[test]
    fn _is_module_loaded() {
        env::set_var("LOADEDMODULES", "blast/12.3:blast/11.1");
        assert_eq!(false, is_module_loaded("", false));
        // FIXME this should be false
        assert_eq!(false, is_module_loaded("bla", false));
        assert_eq!(true, is_module_loaded("blast", false));

        env::set_var(
            "LOADEDMODULES",
            "gcc/x86_64/4.8.2:armadillo/x86_64/4.300.2:igraph/x86_64/0.6.5:python2/x86_64/2.7.2:\
             gcc/x86_64/4.8.2:python/x86_64/3.5.1:",
        );
        assert_eq!(true, is_module_loaded("python", false));
        assert_eq!(true, is_module_loaded("python/x86_64/3.5.1", false));
        assert_eq!(true, is_module_loaded("python2", false));
        env::set_var(
            "LOADEDMODULES",
            "gcc/x86_64/4.8.2:armadillo/x86_64/4.300.2:igraph/x86_64/0.6.5:gcc/x86_64/4.8.2:python/x86_64/3.\
             5.1:",
        );
        assert_eq!(false, is_module_loaded("python2", false));
    }

}
