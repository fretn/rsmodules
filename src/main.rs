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

extern crate rand;
use rand::Rng;
use std::panic;

#[macro_use]
mod macros;

#[macro_use]
extern crate lazy_static;

#[path = "rsmodules.rs"]
mod rsmod;

mod wizard;

use rsmod::Rsmodule;

extern crate bincode;
extern crate dirs;
extern crate rustc_serialize;
extern crate users;
extern crate walkdir;

extern crate ansi_term;
extern crate getopts;
extern crate glob;
extern crate gumdrop;
extern crate gumdrop_derive;
extern crate is_executable;
extern crate mdcat;
extern crate pbr;
extern crate pulldown_cmark;
extern crate regex;
extern crate shellexpand;
extern crate syntect;

use ansi_term::Style;
use std::collections::HashMap;
use std::env;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref TMPFILE_INITIALIZED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref TMPFILE_PATH: Mutex<String> = Mutex::new(String::new());
    static ref OUTPUT_BUFFER: Mutex<Vec<String>> = Mutex::new(vec![]);
}

static CRASH_UNSUPPORTED_SHELL: i32 = 1;
static CRASH_FAILED_TO_CREATE_TEMPORARY_FILE: i32 = 2;
static CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE: i32 = 3;
static CRASH_NO_CACHE_FILES_FOUND: i32 = 4;
static CRASH_MODULE_NOT_FOUND: i32 = 5;
static CRASH_COULDNT_OPEN_CACHE_FILE: i32 = 5;
//static CRASH_NO_ARGS: i32 = 6;
static CRASH_MODULEPATH_IS_FILE: i32 = 7;
static CRASH_CANNOT_ADD_TO_ENV: i32 = 8;
static CRASH_MISSING_INIT_FILES: i32 = 9;
static CRASH_GET_SHELL: i32 = 10;
static CRASH_CREATE_ERROR: i32 = 11;
static CRASH_INVALID_REGEX: i32 = 12;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn is_shell_supported(shell: &str) -> bool {
    // when noshell is selected, all output is printed
    // to stdout instead of the temp file
    // noshell is also useful for debugging purposes

    let shell_list = vec!["tcsh", "csh", "bash", "zsh", "noshell", "python", "perl", "progressbar", "r"];

    if shell_list.contains(&shell) {
        return true;
    }

    false
}

#[cfg(debug_assertions)]
fn release_debug() -> String {
    String::from(" (debug)")
}
#[cfg(not(debug_assertions))]
fn release_debug() -> String {
    String::from("")
}

fn usage(in_eval: bool, subcommand_help: bool) {
    // SubCommandHelp
    let mut sch = HashMap::new();

    // the \t is part of an advanced markup codebase ^^
    // \t doesn't show up in the full usage text
    // but in the partial usage text its replaced by a newline
    // this makes the partial usage text look nicer

    sch.insert("load".to_owned(), "load [(partial) module name(s)]");
    sch.insert(
        "unload".to_owned(),
        "unload [(partial) module name(s)]\t
            A partial module name is the part of the modulename
            before a slash, eg: you have module name 'rsmodules/2.0.0'
            the partial name is 'rsmodules'.",
    );

    sch.insert(
        "switch".to_owned(),
        "switch [(partial) module name from] [(partial) module name to]\t
            Switches between two version of modules.

            This does the same as module load blast/1.2.3 when
            blast/1.2.5 was already loaded.
            This feature was added for compatibility reasons.",
    );

    sch.insert(
        "list".to_owned(),
        "list\t
            Lists all the loaded modules.",
    );

    sch.insert(
        "purge".to_owned(),
        "purge\t
            Unloads all loaded modules.",
    );

    sch.insert(
        "refurbish".to_owned(),
        "refurbish\t
            Unloads all loaded modules. And loads the autoloaded modules.",
    );

    sch.insert(
        "refresh".to_owned(),
        "refresh\t
            Reloads all loaded modules.",
    );

    sch.insert(
        "available".to_owned(),
        "available [--default] [--regex] [search string]\t
            Lists all the available modules.
            If a [search string] is given then all modules which match
            the search string will be listed.
            The search string can also contain multiple items separated
            by spaces.

            When --default, -d is specified then only default modules
            will be listed.

            When --deprecated, -R is specified then only deprecated modules
            will be listed.

            When --regex or -r is specified the search term can be a
            regular expression.",
    );

    sch.insert(
        "info".to_owned(),
        "info [(partial) module name(s)]\t
            Gives more info about a module. Description, which
            variables it modifies and/or which commands are executed
            upon launch.",
    );

    sch.insert(
        "undo".to_owned(),
        "undo\t
            Undo the previous module command, only works for load, unload,
            switch and purge.",
    );

    sch.insert(
        "cache".to_owned(),
        "cache [--help] [make|add|edit|delete]\t
            Manipulate the contents of a .modulesindex file.

            make\t

            Updates the .modulesindex file in all the paths that
            are found in the $MODULEPATH variable. This will only
            work if you have the correct permissions.
            If you want a progress bar use the command:
            update_modules_cache instead of module makecache

            add --modulepath [path] --name [modulename] --description \"description\" [--default] [--deprecated] \t

            Adds a module to the .modulesindex file
            This will only work if you have the correct permissions.
            ",
    );

    sch.insert(
        "addtocache".to_owned(),
        "addtocache [path from $MODULEPATH] [modulename] [default: true/false] [deprecated: true/false ][\"description\"]\t
            Adds a module to the .modulesindex file
            This will only work if you have the correct permissions.",
    );

    sch.insert(
        "makecache".to_owned(),
        "makecache\t
            Updates the .modulesindex file in all the paths that
            are found in the $MODULEPATH variable. This will only
            work if you have the correct permissions.
            If you want a progress bar use the command:
            update_modules_cache instead of module makecache",
    );

    sch.insert(
        "create".to_owned(),
        "create [--help] [modulename]\t
            Starts a wizard to create a modulefile.",
    );

    sch.insert(
        "delete".to_owned(),
        "delete\t
            Deletes a modulefile. As with makecache, this only works
            if you have the correct permissions.",
    );

    sch.insert(
        "autoload".to_owned(),
        "autoload append|prepend|remove|list|purge [module name(s)]\t
            Manages the autoloading of modules when opening a new terminal.",
    );

    sch.insert(
        "readme".to_owned(),
        "readme [(partial)modulename]\t
            Looks for a manpage or a README file in the module installation
            folder and displays the contents of this file.",
    );

    sch.insert(
        "cd".to_owned(),
        "cd [(partial)modulename]\t
            Changes your current working directory to the module
            installation folder. When you don't provide a modulename
            the working directory is changed to the module installation
            folder of the last loaded module.",
    );

    sch.insert(
        "edit".to_owned(),
        "edit [(partial)modulename]\t
            Opens the modulefile in your $EDITOR or if this variable is not
            present in vi -e.",
    );

    let long_help: &str = &format!(
        "

    RSModules manages your user environment on linux and macOS.
    The RSModules package is a tool to help users modifying their environment
    during a session by using modulefiles.
    A modulefile contains all the settings needed to configure the shell for
    using a certain application.

    A modulefile sets or alters environment variables such as PATH,
    LD_LIBRARY_PATH, MANPATH, PYTHONPATH, PERL5LIB, ...

    Modulefiles can be shared by many users or can be used by individuals
    by setting up paths in the MODULEPATH environment variable. Once
    a new modulepath is created and added to MODULEPATH,
    the cache needs to be updated by invoking the command: module makecache.

    Modulefiles can be loaded and unloaded by the user whenever the
    module command is available.

    * module [subcommand] <module name>

        subcommands
        -----------

        * {}
        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}

        * {}
        ",
        help!(sch, "load"),
        help!(sch, "unload"),
        help!(sch, "switch"),
        help!(sch, "list"),
        help!(sch, "purge"),
        help!(sch, "refurbish"),
        help!(sch, "refresh"),
        help!(sch, "available"),
        help!(sch, "info"),
        help!(sch, "undo"),
        help!(sch, "cache"),
        help!(sch, "addtocache"),
        help!(sch, "makecache"),
        help!(sch, "create"),
        help!(sch, "delete"),
        help!(sch, "autoload"),
        help!(sch, "readme"),
        help!(sch, "cd"),
        help!(sch, "edit")
    );

    let error_msg: &str;

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && subcommand_help {
        let (shell, _) = rsmod::get_shell_info();
        eprintln!("");
        eprintln!(
            "  {}: module {}",
            bold(&shell, "Usage"),
            help!(sch, &args[2]).replace("            ", "  ").replace("\t", "\n")
        );
        eprintln!("");
        return;
    }

    eprintln!("  RSModules {}{} - {}", VERSION, release_debug(), AUTHORS);
    eprintln!("");
    eprintln!("  2017 - Ghent University / VIB");
    eprintln!("  http://www.psb.ugent.be - http://www.ugent.be - http://www.vib.be");
    eprintln!("");
    eprintln!("");

    if in_eval {
        error_msg =
            "  Usage: module \
             <load|unload|list|switch|purge|refurbish|refresh|available|undo|info|addtocache|makecache|delete|autoload|readme|cd|edit> [module \
             name]";
    } else {
        error_msg =
            "  Usage: rsmodules <shell> \
             <load|unload|list|switch|purge|refurbish|refresh|available|undo|info|addtocache|makecache|delete|autoload|readme|cd|edit> [module \
             name]";
    }

    eprintln!("{}", &error_msg);
    if !in_eval {
        eprintln!(
            "  Supported shells: bash, zsh, csh, tcsh, python, perl and \
             noshell"
        );
        eprintln!("");
        eprintln!("  When noshell is selected all output is printed to stdout,");
        eprintln!("  module available will then print a nice list without gaps,");
        eprintln!("  which makes your life easier when you want to parse this output.");
    }
    eprintln!("{}", &long_help);
}

fn set_global_tmpfile(tmp_file_path: String) {
    let mut tmp = lu!(TMPFILE_PATH);
    *tmp = tmp_file_path;

    TMPFILE_INITIALIZED.store(true, Ordering::Relaxed);
}

fn run(args: &[String]) {
    let command: &str;
    let tmp: String;
    let mut modulename: &str = "";

    let (shell, shell_width) = rsmod::get_shell_info();

    ////

    if !is_shell_supported(&shell) {
        usage(false, false);
        rsmod::crash(CRASH_UNSUPPORTED_SHELL, &format!("{} is not a supported shell", shell));
    }

    let modulepaths = rsmod::get_module_paths(false);

    // create temporary file in the home folder
    // if the file cannot be created try to create it
    // in /tmp, if that fails, the program exits
    //
    // ~/.rsmodulestmpXXXXXXXX
    // /tmp/.rsmodulestmpXXXXXXXX

    let mut tmpfile: File;

    let rstr: String = rand::thread_rng().gen_ascii_chars().take(8).collect();

    let mut tmp_file_path: PathBuf;

    match dirs::home_dir() {
        Some(path) => tmp_file_path = path,
        None => {
            show_warning!(
                "We were unable to find your home directory, checking if /tmp is an \
                 option"
            );

            // this is wrong, as we try to use temp again a bit later
            tmp_file_path = env::temp_dir();
            // return;
        }
    };

    let filename: String = format!(".rsmodulestmp{}", rstr);
    let filename: &str = filename.as_ref();
    tmp_file_path.push(filename);

    match File::create(&tmp_file_path) {
        Ok(file) => {
            tmpfile = file;
            set_global_tmpfile(tmp_file_path.to_str().unwrap().to_string());
        }
        Err(_) => {
            // home exists but we can't create the temp file in it or
            // worst case, /tmp exists but we can't create the temp file in it
            tmp_file_path = env::temp_dir();
            let filename: String = format!(".rsmodulestmp{}", rstr);
            let filename: &str = filename.as_ref();
            tmp_file_path.push(filename);

            match File::create(&tmp_file_path) {
                Ok(newfile) => {
                    tmpfile = newfile;
                    set_global_tmpfile(tmp_file_path.to_str().unwrap().to_string());
                }
                Err(e) => {
                    rsmod::crash(
                        CRASH_FAILED_TO_CREATE_TEMPORARY_FILE,
                        &format!("Failed to create temporary file: {}", e),
                    );
                    return;
                }
            };
        }
    };

    panic::set_hook(Box::new(|_| {
        let tmp = lu!(TMPFILE_PATH);
        let tmp_file_path = &*tmp;
        remove_file(tmp_file_path).unwrap();
    }));

    let filename = tmp_file_path.to_str().unwrap().to_string();

    let mut quoted_string: String;
    let mut command_hit: &str = "";
    if args.len() >= 3 {
        command = &args[2];
        let matches: bool;
        let mut modulenames: Vec<String> = Vec::new();
        if args.len() > 3 {
            for arg in args.iter().skip(3) {
                let whitespace: Vec<&str> = arg.split_whitespace().collect();
                if whitespace.len() > 1 {
                    quoted_string = format!("\"{}\"", arg);
                    modulenames.push(quoted_string);
                } else {
                    modulenames.push(arg.clone());
                }
            }
            //modulename = &args[3];
            tmp = modulenames.join(" ");
            modulename = &tmp;
        }

        let mut command_list: Vec<&str> = Vec::new();
        command_list.push("load");
        command_list.push("add");
        command_list.push("unload");
        command_list.push("rm");
        command_list.push("available");
        command_list.push("list");
        command_list.push("purge");
        command_list.push("refurbish");
        command_list.push("refresh");
        command_list.push("info");
        command_list.push("display");
        command_list.push("show");
        command_list.push("switch");
        command_list.push("cache");
        command_list.push("addtocache");
        command_list.push("makecache");
        command_list.push("help");
        command_list.push("undo");
        command_list.push("autoload");
        command_list.push("readme");
        command_list.push("delete");
        command_list.push("create");
        command_list.push("cd");
        command_list.push("edit");
        command_list.push("--help");
        command_list.push("-h");
        // TODO
        // "create" -> wizard to create a new mdoule
        // "addmodulepath" -> wizard to add a path to $MODULEPATH
        // "removemodulepath" -> wizard to remove a path from $MODULEPATH
        //  ask to update /etc/profile.d or bashrc or personal_cshrc
        // "delete" -> deletes a modulefile
        // "update" -> when you have blast/12.3 as module
        //  module update blast 13.3 or module update blast/12.3 13.3
        //  will copy that module file to a new file blast/13.3
        //  and it will replace all instances of 12.3 in the file with
        //  13.3
        //

        if command == "help" || command == "--help" || command == "-h" {
            usage(true, false);
            return;
        }

        let mut num_hits: i32 = 0;

        for cmd in command_list {
            if cmd.starts_with(command) {
                num_hits += 1;
                command_hit = cmd;
            }
        }

        let loadedmodules: String;
        if num_hits != 1 {
            usage(true, false);
            return;
        } else {
            matches = true;

            if command_hit == "cd" {
                modulename = if modulename.is_empty() {
                    match env::var(rsmod::ENV_LOADEDMODULES) {
                        Ok(list) => loadedmodules = list,
                        Err(_) => {
                            loadedmodules = String::from("");
                        }
                    };

                    let mut loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
                    loadedmodules.retain(|&x| x != "");

                    let loadedmodule: &str = if !loadedmodules.is_empty() { loadedmodules[0] } else { "" };
                    loadedmodule
                } else {
                    modulename
                };
            }

            if command_hit == "add" {
                command_hit = "load";
            }
            if command_hit == "rm" {
                command_hit = "unload";
            }
            if command_hit == "display" || command_hit == "show" {
                command_hit = "info";
            }

            if command_hit == "load" || command_hit == "unload" {
                // undo doesn't work for dependency loaded modules
                let data = setenv(
                    "RSMODULES_UNDO",
                    &format!("{} {}", command_hit, modulename.to_string()),
                    &shell,
                );
                crash_cleanup_if_err!(
                    CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                    tmpfile.write_all(data.as_bytes()),
                    filename
                );
            }

            if (command_hit == "load"
                || command_hit == "unload"
                || command_hit == "info"
                || command_hit == "delete"
                || command_hit == "readme"
                || command_hit == "edit")
                && args.len() == 3
            {
                usage(true, true);
                return;
            }

            if command_hit == "switch" && args.len() != 5 {
                usage(true, true);
                return;
            }

            if command_hit == "switch" {
                modulenames.reverse();
                let data = setenv(
                    "RSMODULES_UNDO",
                    &format!("{} {}", command_hit, modulenames.join(" ")),
                    &shell,
                );
                crash_cleanup_if_err!(
                    CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                    tmpfile.write_all(data.as_bytes()),
                    filename
                );
            }

            if command_hit == "purge" {
                let loaded_list = rsmod::get_loaded_list();
                let mut args: Vec<String> = Vec::new();
                for (argument, _, _) in loaded_list {
                    args.push(argument);
                }
                let loadedmodules = args.join(" ");
                let data = setenv("RSMODULES_UNDO", &format!("unload {}", loadedmodules), &shell);
                crash_cleanup_if_err!(
                    CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                    tmpfile.write_all(data.as_bytes()),
                    filename
                );
            }

            let mut rsmod_command: Rsmodule = Rsmodule {
                cmd: command_hit,
                typed_command: command,
                arg: modulename,
                search_path: &modulepaths,
                shell: &shell,
                //shell_width: shell_width,
                shell_width,
            };
            rsmod::command(&mut rsmod_command);
        }

        if !matches {
            usage(false, false);
        }
    }

    // when noshell is choosen, we just output to stdout
    // this is used for scripts that want to parse the module av output
    // for example for tab completion

    if shell != "noshell" && shell != "python" && shell != "perl" && shell != "progressbar" && shell != "r" {
        // we want a self destructing tmpfile
        // so it must delete itself at the end of the run
        // if it crashes it will be deleted after the source stuff
        // if the code that writes the file crashes it should clean up

        let cmd = format!("\\rm -f {}\n", tmp_file_path.display());

        let mut output_buffer = lu!(OUTPUT_BUFFER);
        let output_buffer = &mut (*output_buffer);
        output_buffer.push(cmd);

        for line in output_buffer {
            crash_cleanup_if_err!(
                CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                tmpfile.write_all(line.as_bytes()),
                filename
            );
        }

        // source tmpfile
        println!("source {}", tmp_file_path.display());
        // doesn't this make more sense than creating a
        // self destructing file ?
        println!("rm -f {}", tmp_file_path.display());
    } else {
        remove_file(tmp_file_path.to_str().unwrap().to_string()).unwrap();
    }
}

pub fn setenv(var: &str, val: &str, shell: &str) -> String {
    let mut data: String = String::new();
    if shell == "bash" || shell == "zsh" {
        data = format!("export {}=\"{}\"\n", var, val);
    } else if shell == "tcsh" || shell == "csh" {
        data = format!("setenv {} \"{}\"\n", var, val);
    } else if shell == "python" {
        data = format!("os.environ[\"{}\"] = \"{}\";\n", var, val);
    } else if shell == "r" {
        data = format!(
            "old_path <- Sys.getenv(\"{}\")

        Sys.setenv({} = paste(old_path, \"{}\", sep = \":\"))",
            var, var, val
        );
    } else if shell == "perl" {
        data = format!("$ENV{{{}}}=\"{}\";\n", var, val);
    }

    data
}

fn bold<'a>(shell: &str, msg: &'a str) -> ansi_term::ANSIGenericString<'a, str> {
    if shell == "noshell"
        || shell == "perl"
        || shell == "r"
        || shell == "python"
        || env::var("TERM") == Ok(String::from(""))
        || env::var("NO_COLOR").is_ok()
    {
        return Style::new().paint(msg);
    }

    Style::new().bold().paint(msg)
}

pub fn output(line: String) {
    let mut output_buffer = lu!(OUTPUT_BUFFER);
    let output_buffer = &mut (*output_buffer);
    output_buffer.push(line);
}

fn init() {
    TMPFILE_INITIALIZED.store(false, Ordering::Relaxed);
}

fn main() {
    init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        if !wizard::run(false) {
            usage(false, false);
        }
        return;
    }

    if args.len() == 2 {
        usage(true, false);
    }

    if args.len() >= 2 && (args.get(1) == Some(&String::from("-h")) || args.get(1) == Some(&String::from("--help"))) {
        usage(false, false);
        return;
    }

    run(&args);
}

#[cfg(test)]
mod tests {
    use super::is_shell_supported;

    #[test]
    fn supported_shells() {
        assert_eq!(false, is_shell_supported("randomshellname"));
        assert_eq!(true, is_shell_supported("bash"));
        assert_eq!(true, is_shell_supported("zsh"));
        assert_eq!(true, is_shell_supported("tcsh"));
        assert_eq!(true, is_shell_supported("csh"));
        assert_eq!(true, is_shell_supported("r"));
    }
}
