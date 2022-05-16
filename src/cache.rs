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

use super::super::bold;
use chrono::{DateTime, Utc};
use if_let_return::if_let_some;
use regex::Regex;
use std::cmp::Ordering;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Stdout};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use walkdir::WalkDir;
extern crate bincode;
use super::{crash, echo, get_module_description, get_module_paths, is_module_loaded, AvailableOptions, Rsmodule};
use bincode::rustc_serialize::{decode_from, encode_into};

use super::script;

use pbr::ProgressBar;

use gumdrop::Options;

pub static MODULESINDEX: &str = ".modulesindex";

#[derive(RustcEncodable, RustcDecodable, Clone, Eq, Debug)]
struct Module {
    name: String,
    description: String,
    default: bool,
    deprecated: String,
}

impl Module {
    pub fn new() -> Module {
        Module {
            name: String::new(),
            description: String::new(),
            default: false,
            deprecated: String::from("0"),
        }
    }

    pub fn from(name: String, description: String, default: bool, deprecated: String) -> Module {
        Module {
            name,
            description,
            default,
            deprecated,
        }
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Module) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Module) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Module) -> bool {
        self.name == other.name
    }
}

#[cfg(debug_assertions)]
pub fn release_debug() -> String {
    String::from("_debug")
}
#[cfg(not(debug_assertions))]
pub fn release_debug() -> String {
    String::from("")
}

fn add_module(name: String, description: String, default: bool, deprecated: String, modules: &mut Vec<Module>) {
    let module: Module = Module::from(name, description, default, deprecated);

    modules.push(module);
}

fn get_default_version(modulepath: &str, modulename: &str) -> String {
    let parts: Vec<&str> = modulename.split('/').collect();
    let groupname = if !parts.is_empty() { parts[0] } else { "" };

    let tmp = format!("{}/{}/.version", modulepath, groupname);
    let module_path = Path::new(&tmp);

    // read filename line by line, and push it to modules
    let mut buffer = String::new();

    if Path::new(&module_path).is_file() {
        let file: File = match File::open(module_path) {
            Ok(file) => file,
            Err(_) => {
                return String::from("");
            }
        };

        let file = BufReader::new(file);
        // if there are multiple entries in .version, the last one counts
        for (_, line) in file.lines().enumerate() {
            buffer = line.unwrap();
        }
    }

    buffer
}

fn is_default_version(modulepath: &str, modulename: &str) -> bool {
    let parts: Vec<&str> = modulename.split('/').collect();
    let groupname = if !parts.is_empty() { parts[0] } else { "" };

    let tmp = format!("{}/{}/.version", modulepath, groupname);
    let module_path = Path::new(&tmp);

    // read filename line by line, and push it to modules
    let mut buffer = String::new();

    if Path::new(&module_path).is_file() {
        let file: File = match File::open(module_path) {
            Ok(file) => file,
            Err(_) => {
                return false;
            }
        };

        let file = BufReader::new(file);
        // if there are multiple entries in .version, the last one counts
        for (_, line) in file.lines().enumerate() {
            buffer = line.unwrap();
        }
    }

    if buffer == modulename {
        return true;
    }

    false
}

fn progressbar(num: u64, msg: &str) -> ProgressBar<Stdout> {
    let mut pb = ProgressBar::new(num);
    pb.show_speed = false;
    pb.show_time_left = false;
    pb.show_counter = false;
    pb.show_message = true;
    pb.message(msg);

    pb
}

pub fn add_module_to_index(
    modulepath: &str,
    shell: &str,
    name: &str,
    description: &str,
    default: &str,
    deprecated: &str,
) -> bool {
    if modulepath.is_empty() {
        return false;
    }
    // TODO: check if the modulefile exists, if not don't add it

    let file_str = format!("{}/{}{}", modulepath, MODULESINDEX, release_debug());
    let file: File = match File::open(&file_str) {
        Ok(file) => file,
        Err(_) => {
            if shell != "noshell" {
                echo("", shell);
                let msg: String = format!(
                    "  {}: {} could NOT be opened.",
                    bold(shell, "WARNING"),
                    bold(shell, modulepath)
                );
                echo(&msg, shell);
            } else {
                let msg: String = format!("{} failed", modulepath);
                echo(&msg, shell);
            }
            return false;
        }
    };

    let default = if default == "true" { true } else { false };
    //let deprecated = if deprecated == "true" { "1" } else { "0" };

    let mut reader = BufReader::new(&file);
    let mut modules: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite).unwrap();

    // open the file again
    let file: File = match File::options().write(true).open(&file_str) {
        Ok(file) => file,
        Err(_) => {
            if shell != "noshell" {
                echo("", shell);
                let msg: String = format!(
                    "  {}: {} could NOT be opened.",
                    bold(shell, "WARNING"),
                    bold(shell, modulepath)
                );
                echo(&msg, shell);
            } else {
                let msg: String = format!("{} failed", modulepath);
                echo(&msg, shell);
            }
            return false;
        }
    };

    let module: Module = Module::from(name.to_string(), description.to_string(), default, deprecated.to_string());

    if module.default {
        let default_version = get_default_version(modulepath, &module.name);
        for tmp_module in modules.iter_mut() {
            if tmp_module.name == default_version {
                tmp_module.default = false;
            }
        }
    }

    // this only checks the modulename, so you cannot overwrite
    if !modules.contains(&module) {
        let mut writer = BufWriter::new(&file);
        modules.push(module);
        encode_into(&modules, &mut writer, bincode::SizeLimit::Infinite).unwrap();
    }

    return true;
}

pub fn update(modulepath: &str, shell: &str) -> bool {
    // list is: path to file, module name, default
    let mut list: Vec<(String, String, bool, script::Deprecated)> = Vec::new();
    let module_path = Path::new(&modulepath);
    let mut index_succes: i32 = 0;
    let mut index_default: i32 = 0;

    let file_str = format!("{}/{}{}", modulepath, MODULESINDEX, release_debug());
    let num_modules = if Path::new(&file_str).exists() {
        count_modules_in_cache(&PathBuf::from(&file_str))
    } else {
        0
    };

    if shell == "progressbar" {
        echo("", shell);
        echo(&format!("  Indexing {}", modulepath), shell);
        echo("", shell);
    }

    let mut pb = if num_modules != 0 && shell == "progressbar" {
        progressbar(num_modules, "  Scanning folders ")
    } else {
        ProgressBar::new(0)
    };

    #[allow(clippy::redundant_closure)]
    for entry in WalkDir::new(module_path).into_iter().filter_map(|e| e.ok()) {
        let str_path: &str = entry.path().to_str().unwrap();

        let part: Vec<&str> = str_path.split(&modulepath).collect();

        if !entry.path().is_dir() {
            for mut modulename in part {
                if modulename != "" {
                    if shell == "progressbar" {
                        pb.inc();
                    }
                    let first = modulename.chars().next().unwrap();
                    let second = if modulename.len() >= 2 { &modulename[1..2] } else { "" };

                    if modulename == format!("{}", MODULESINDEX) {
                        continue;
                    }

                    if modulename == format!("{}_debug", MODULESINDEX) {
                        continue;
                    }
                    // modulename can start with /
                    // also we don't want the .modulesindex or other hidden files
                    if first == '/' && second != "." {
                        modulename = modulename.trim_start_matches('/');
                    }

                    let modulename_part: Vec<&str> = modulename.split('/').collect();
                    let mut is_version_file = false;

                    // skip the .version files, these are not modulefiles
                    for mp in modulename_part {
                        if mp == ".version" {
                            is_version_file = true;
                        }
                    }

                    if second != "." && !is_version_file {
                        // check if this module is deprecated or not
                        let path = format!("{}/{}", modulepath, modulename);
                        let path = PathBuf::from(&path);

                        script::run(&path, "deprecated");
                        let deprecated;

                        {
                            // we ne need a different scope, or DEPRECATED is locked
                            let tmp_deprecated = lu!(script::DEPRECATED);
                            deprecated = tmp_deprecated.clone();
                        }

                        //
                        let default = is_default_version(modulepath, modulename);
                        list.push((str_path.to_string(), modulename.to_string(), default, deprecated.clone()));
                        index_succes += 1;
                    }
                }
            }
        }
    }

    // now we have all the module files in the current folder
    // we need to parse them to get their description
    // our list of modules that we will save into the .modulesindex

    let mut modules: Vec<Module> = vec![];

    let num_modules = list.len() as u64;
    let mut pb = if num_modules != 0 && shell == "progressbar" {
        progressbar(num_modules, "  Parsing files    ")
    } else {
        ProgressBar::new(0)
    };

    for (modulepath, modulename, default, deprecated) in list {
        let path: PathBuf = PathBuf::from(&modulepath);
        if shell == "progressbar" {
            pb.inc();
            pb.message("  Parsing files    ");
        }

        let description: Vec<String> = get_module_description(&path, "description");
        let description = description.join(" ");

        // flags is supposed to be a bitfield
        // currently it is only used for flagging a module as default
        //        let mut _default: bool = false;
        if default {
            //           flags = flags | ModuleFlags::DEFAULT;
            index_default += 1;
        }

        match deprecated.state {
            script::DeprecatedState::Not => add_module(modulename, description, default, "0".to_string(), &mut modules),
            script::DeprecatedState::Before => add_module(modulename, description, default, deprecated.time, &mut modules),
            script::DeprecatedState::After => add_module(modulename, description, default, deprecated.time, &mut modules),
            //script::DeprecatedState::After => {}
        };
    }

    if shell == "progressbar" {
        echo("", shell);
    }
    echo("", shell);
    echo(&"  Writing cache file.", shell);

    let file: File = match File::create(&file_str) {
        Ok(file) => file,
        Err(_) => {
            if shell != "noshell" {
                echo("", shell);
                let msg: String = format!(
                    "  {}: {} could NOT be indexed.",
                    bold(shell, "WARNING"),
                    bold(shell, modulepath)
                );
                echo(&msg, shell);
            } else {
                let msg: String = format!("{} failed", modulepath);
                echo(&msg, shell);
            }
            return false;
        }
    };

    let mut writer = BufWriter::new(file);
    encode_into(&modules, &mut writer, bincode::SizeLimit::Infinite).unwrap();

    if shell != "noshell" {
        echo("", shell);
        let msg: String = format!("  {} was succesfully indexed.", bold(shell, modulepath));
        echo(&msg, shell);
        echo("", shell);
        let tmp = format!("{}", index_succes);
        let msg: String = format!("  * Total number of modules: {}", bold(shell, &tmp));
        echo(&msg, shell);
        let tmp = format!("{}", index_default);
        let msg: String = format!("  * Number of default (D) modules: {}", bold(shell, &tmp));
        echo(&msg, shell);
        echo("", shell);
    } else {
        let msg: String = format!("{} success", modulepath);
        echo(&msg, shell);
        let tmp = format!("{}", index_succes);
        let msg: String = format!("Total number of modules: {}", &tmp);
        echo(&msg, shell);
        let tmp = format!("{}", index_default);
        let msg: String = format!("Number of default (D) modules: {}", &tmp);
        echo(&msg, shell);
    }

    true
}

fn count_modules_in_cache(filename: &PathBuf) -> u64 {
    let file: File = match File::open(filename) {
        Ok(file) => file,
        Err(_) => {
            crash(
                super::super::CRASH_COULDNT_OPEN_CACHE_FILE,
                &format!(
                    "count_modules_in_cache: couldn't open the required index file: {:?}",
                    filename
                ),
            );
            return 0;
        }
    };
    let mut reader = BufReader::new(file);
    let decoded: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite).unwrap();

    decoded.len() as u64
}

pub fn parse_modules_cache_file(filename: &PathBuf, modules: &mut Vec<(String, bool, String)>) {
    let file: File = match File::open(filename) {
        Ok(file) => file,
        Err(_) => {
            crash(
                super::super::CRASH_COULDNT_OPEN_CACHE_FILE,
                &format!(
                    "parse_modules_in_cache: couldn't open the required index file: {:?}",
                    filename
                ),
            );
            return;
        }
    };
    let mut reader = BufReader::new(file);
    let decoded: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite).unwrap();

    for module in decoded {
        modules.push((module.name, module.default, module.deprecated));
    }
}

fn find_char_boundary(s: &str, i: usize) -> Option<usize> {
    if i > s.len() {
        return Some(i);
    }

    let mut end = i;
    while !s.is_char_boundary(end) {
        end += 1;
    }
    Some(end)
}

pub fn get_module_list(arg: &str, rsmod: &Rsmodule, opts: &AvailableOptions) {
    let typed_command: &str = &rsmod.typed_command;
    let shell: &str = &rsmod.shell;
    let shell_width: usize = rsmod.shell_width;
    let modulepaths = get_module_paths(false);

    let re: Regex = match Regex::new(arg) {
        Ok(re) => re,
        Err(_) => {
            crash(super::super::CRASH_INVALID_REGEX, "Invalid regular expression");
            return;
        }
    };

    // prints a nice list for module av
    // no gaps, no default, no description
    // usefull for parsing, eg for bash completion
    let simple_list = shell == "noshell" || shell == "python" || shell == "perl";

    let mut longest_name = 0;
    let mut decoded: Vec<Module> = Vec::new();
    for modulepath in modulepaths.clone() {
        let file: File = match File::open(format!("{}/{}{}", modulepath, MODULESINDEX, release_debug())) {
            Ok(file) => file,
            Err(_) => {
                echo(
                    &format!("  {}: {} doesn't contain an index.", bold(shell, "WARNING"), modulepath),
                    shell,
                );
                if update(&modulepath, shell) {
                    match File::open(format!("{}/{}{}", modulepath, MODULESINDEX, release_debug())) {
                        Ok(file) => file,
                        Err(_) => {
                            // for some unknown reason we cannot open the file
                            // while we could generate it in the update function
                            // let's be honest: we're screwed
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }
        };

        let mut reader = BufReader::new(file);
        let decoded_file: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite).unwrap();
        for item in decoded_file {
            decoded.push(item);
        }

        if arg != "" {
            for module in decoded.clone() {
                let avmodule_lc: String = module.name.to_lowercase();
                let module_lc: String = arg.to_lowercase();
                let avmodule_lc: &str = avmodule_lc.as_ref();
                let module_lc: &str = module_lc.as_ref();

                let matches: bool = if opts.regex {
                    re.is_match(&module.name)
                } else {
                    avmodule_lc.contains(module_lc)
                };

                if longest_name <= module.name.len() && matches {
                    longest_name = module.name.len();
                }
            }
            longest_name += 1;
        } else {
            for module in decoded.clone() {
                if longest_name <= module.name.len() {
                    longest_name = module.name.len();
                }
            }
            longest_name += 1;
        }
    }

    decoded.sort_by(|a, b| natord::compare(&a.name.as_str(), &b.name.as_str()));

    let mut previous_first_char: char = '§';
    let mut previous_description: String = String::new();
    let mut cnt = 0;
    for module in decoded {
        let tmp: String;

        let mut description = module.description.clone();
        let position = shell_width - longest_name - 5;
        let position = match find_char_boundary(&description, position) {
            Some(p) => p,
            None => 0,
        };

        //description.truncate(shell_width - longest_name - 5);
        description.truncate(position);

        if module.description == previous_description {
            description = String::new();
        }
        previous_description = module.description;

        let mut deprecated = " ";
        if module.deprecated != "0" {
            let now = Utc::now().timestamp_millis();

            let mstime = format!("{} 00:00:00 +0000", module.deprecated);
            let mstime = match DateTime::parse_from_str(&mstime, "%Y-%m-%d %T %z") {
                Ok(mstime) => mstime,
                Err(e) => {
                    eprintln!("Error parsing deprecated time argument: {}", e);
                    break;
                }
            };
            let mstime = mstime.timestamp_millis();

            if now > mstime {
                deprecated = "R";
            } else {
                deprecated = "#";
            }
        }
        let default = if module.default == true { "D" } else { deprecated };

        if opts.default && module.default != true {
            continue;
        }

        if opts.deprecated && module.deprecated == "0" {
            continue;
        }

        if simple_list {
            tmp = module.name.clone();
        } else if is_module_loaded(module.name.as_ref(), true) {
            let tmpwidth = if module.name.len() < longest_name {
                longest_name - module.name.len()
            } else {
                0
            };

            tmp = format!(
                "{} {}{:width$} | {}",
                default,
                bold(shell, &module.name),
                " ",
                description,
                width = tmpwidth
            );
        } else {
            tmp = format!("{} {:width$} | {}", default, module.name, description, width = longest_name);
        }

        if arg != "" {
            let avmodule_lc: String = module.name.to_lowercase();
            let module_lc: String = arg.to_lowercase();
            let avmodule_lc: &str = avmodule_lc.as_ref();
            let module_lc: &str = module_lc.as_ref();

            let matches: bool = if opts.regex {
                re.is_match(&module.name)
            } else {
                avmodule_lc.contains(module_lc)
            };

            if matches {
                cnt += 1;
                let first_char: char = module.name.chars().next().unwrap();
                if first_char != previous_first_char && !simple_list {
                    // add a newline
                    if cnt == 1 {
                        let width = if longest_name > 12 { longest_name - 12 } else { longest_name };
                        echo("", shell);
                        echo(
                            &format!(
                                "  {} {:width$} | {}",
                                bold(shell, "Module name"),
                                " ",
                                bold(shell, "Description"),
                                width = width
                            ),
                            shell,
                        );
                        echo(&format!("  {:width$} |", " ", width = longest_name), shell);
                    } else {
                        echo(&format!("  {:width$} |", " ", width = longest_name), shell);
                    }
                }
                previous_first_char = module.name.chars().next().unwrap();
                if simple_list {
                    println!("{}", &tmp);
                } else {
                    echo(&tmp, shell);
                }
            }
        } else {
            cnt += 1;
            let first_char: char = module.name.to_lowercase().chars().next().unwrap();
            if first_char != previous_first_char && !simple_list {
                // add a newline
                if cnt == 1 {
                    let width = if longest_name > 12 { longest_name - 12 } else { longest_name };
                    echo("", shell);
                    echo(
                        &format!(
                            "  {} {:width$} | {}",
                            bold(shell, "Module name"),
                            " ",
                            bold(shell, "Description"),
                            width = width
                        ),
                        shell,
                    );
                    echo(&format!("  {:width$} |", " ", width = longest_name), shell);
                } else {
                    echo(&format!("  {:width$} |", " ", width = longest_name), shell);
                }
            }
            previous_first_char = module.name.to_lowercase().chars().next().unwrap();
            if simple_list {
                println!("{}", &tmp);
            } else {
                echo(&tmp, shell);
            }
        }
    }

    if shell != "noshell" && cnt != 0 {
        if cnt > 50 {
            echo("", shell);
            echo("", shell);
            let module = if arg != "" {
                let tmp = format!(" {}", arg);
                tmp.clone()
            } else {
                arg.to_string()
            };
            let space = if module.is_empty() { "" } else { " " };

            let msg = format!("module {} {}{}| more", typed_command.trim(), module.trim(), space);
            echo(
                &format!(
                    "  Hint: use the command '{}' to run the output through a \
                     pager.",
                    bold(shell, &msg)
                ),
                shell,
            );
        } else {
            echo("", shell);
        }

        echo("", shell);
        echo(
            &format!("  {} D means that the module is set as the default module.", bold(shell, "*")),
            shell,
        );
        echo(
            &format!(
                "  {} # means that the module is marked as deprecated and will be removed in the future.",
                bold(shell, "*")
            ),
            shell,
        );
        echo(
            &format!(
                "  {} R means that the module is deprecated and has been removed.",
                bold(shell, "*")
            ),
            shell,
        );
        echo(
            &format!(
                "\n  {} Loaded modules are printed in {}.",
                bold(shell, "*"),
                bold(shell, "bold")
            ),
            shell,
        );
        echo("", shell);
    }
}

// Options accepted for the `make` command
#[derive(Debug, Options)]
struct AddOpts {
    #[options(help = "Print this help message")]
    help: bool,

    #[options(no_short, help = "Single path from $MODULEPATH")]
    modulepath: String,

    #[options(no_short, help = "Name of the module")]
    name: String,

    #[options(no_short, help = "Description of the module")]
    description: String,

    #[options(no_short, help = "Mark module as default", count)]
    default: u8,

    #[options(no_short, help = "Mark module as deprecated, expects date format: YYYY-MM-DD")]
    deprecated: String,
}

// Options accepted for the `make` command
#[derive(Debug, Options)]
struct MakeOpts {
    #[options(help = "Print this help message")]
    help: bool,
}

#[derive(Debug, Options)]
enum Command {
    #[options(help = "Add modules to the cache")]
    Add(AddOpts),
    #[options(help = "Rebuild the cache file")]
    Make(MakeOpts),
}

#[derive(Debug, Default, Options)]
struct CacheOptions {
    #[options(command)]
    command: Option<Command>,

    #[options(help = "Print this help message")]
    help: bool,
}

fn print_help(args: &[String], shell: &str, command: &str) {
    let mut cmd = command;
    if command.is_empty() {
        cmd = "[make|add|edit|delete]";
    }

    if shell == "noshell" || shell == "python" || shell == "perl" {
        eprintln!("Usage: {} cache {} [ARGUMENTS]", args[0], cmd);
    } else {
        eprintln!("Usage: module cache {} [ARGUMENTS]", cmd);
    }
    eprintln!("");
    if cmd == "add" {
        eprintln!("{}", AddOpts::usage());
    } else {
        eprintln!("{}", CacheOptions::usage());
    }
}

pub fn run(rsmod: &Rsmodule) {
    // the module bash function eats quotes
    // so a description with spaces that is quoted doesn't
    // work at all, this is a hack to work around this problem
    let args: Vec<String> = args().collect();
    let mut description_found = false;
    let mut description: Vec<String> = vec![];
    let mut result_args: Vec<String> = vec![];
    for arg in args.iter() {
        if arg == "--description" {
            description_found = true;
        } else if description_found {
            if arg.starts_with("--") {
                description_found = false;
                result_args.push(arg.to_string());
            } else {
                description.push(arg.to_string());
            }
        } else {
            result_args.push(arg.to_string());
        }
    }

    let description = description.join(" ");
    if !description.is_empty() {
        result_args.push("--description".to_string());
        result_args.push(description);
    }
    let args = result_args;
    // end hack

    // Remember to skip the first arguments. That's the program name, shell and command
    let opts = match CacheOptions::parse_args_default(&args[3..]) {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("{}: {}", args[0], e);
            return;
        }
    };

    if opts.command.is_none() {
        print_help(&args, rsmod.shell, "");
        return;
    }
    if_let_some!(command = opts.command_name(), ());

    if command == "make" {
        let modulepaths = get_module_paths(false);
        for modulepath in modulepaths {
            if modulepath != "" {
                update(&modulepath, rsmod.shell);
            }
        }
        return;
    } else if command == "add" {
        let addopts = match AddOpts::parse_args_default(&args[4..]) {
            Ok(opts) => opts,
            Err(e) => {
                eprintln!("{}: {}", args[0], e);
                return;
            }
        };

        let default = if addopts.default > 0 { "true" } else { "false" };
        let deprecated = if addopts.deprecated.is_empty() {
            String::from("0")
        } else {
            // TODO: cleanup this mess
            let mstime = format!("{} 00:00:00 +0000", addopts.deprecated);
            let mstime = DateTime::parse_from_str(&mstime, "%Y-%m-%d %T %z");

            if mstime.is_ok() {
                let mstime = mstime.unwrap();

                let mstime = mstime.date().to_string();
                let mstime = &mstime[0..10];
                String::from_str(mstime).unwrap()
            } else {
                eprintln!("Failed parsing deprecated date, module will not be flagged as deprecated");
                String::from("0")
            }
        };

        let modulepath = addopts.modulepath;
        let name = addopts.name;
        let description = addopts.description;
        if !name.is_empty() && !modulepath.is_empty() {
            add_module_to_index(&modulepath, rsmod.shell, &name, &description, default, &deprecated);
        } else {
            print_help(&args, rsmod.shell, command);
        }

        return;
    }

    if opts.help {
        print_help(&args, rsmod.shell, "");
    }
}
