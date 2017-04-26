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

#[macro_use]
mod macros;

#[macro_use]
extern crate lazy_static;

#[path = "rsmodules.rs"]
mod rsmod;

#[path = "wizard.rs"]
mod wizard;

use rsmod::Rsmodule;

extern crate rustc_serialize;
extern crate bincode;
extern crate walkdir;
extern crate users;

extern crate shellexpand;
extern crate regex;

use std::io::Write;
use std::fs::{File, remove_file};
use std::path::PathBuf;
use std::env;
use std::sync::Mutex;

lazy_static! {
    static ref TMPFILE_INITIALIZED: Mutex<bool> = Mutex::new(false);
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

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

static LONG_HELP: &'static str = "

  RSModules manages your user environment on linux and macOS.
  The RSModules package is a tool to help users modifying their environment
  during a session by using modulefiles.
  A modulefile contains all the settings needed to configure the shell for
  using a certain application.

  A modulefile sets or alters environment variables such as PATH,
  LD_LIBRARY_PATH, MANPATH, PYTHONPATH, PERL5LIB, ...

  Modulefiles can be shared by many users or can be used by individuals
  by setting up paths in the MODULEPATH environment variable. Once
  a modulepath is added, the cache needs to be updated by invoking
  the command: module makecache.

  Modulefiles can be loaded and unloaded by the user whenever the
  module command is available.

  * module [subcommand] <module name>

    subcommands
    -----------

    * load [(partial) module name(s)]
    * unload [(partial) module name(s)]

      A partial module name is the part of the modulename
      before a slash, eg: you have module name 'rsmodules/2.0.0'
      the partial name is 'rsmodules'.

    * switch [(partial) module name from] [(partial) module name to]
      Switches between two version of modules.

      This does the same as module load blast/1.2.3 when
      blast/1.2.5 is loaded.
      This feature was added for compatibility reasons.

    * list
      Lists all the loaded modules.

    * purge
      Unloads all loaded modules.

    * refresh
      Reloads all loaded modules.

    * available <search string>
      Lists all the available modules.
      If a <search string> is given then all modules which match
      the search string will be listed.

    * info [(partial) module name(s)]
      Gives more info about a module. Description, which
      variables it modifies and/or which commands are executed
      upon launch.

    * undo
      Undo the previous module command, only works for load, unload,
      switch and purge.

    * makecache
      Updates the .modulesindex file in all the paths that
      are found in the $MODULEPATH variable. This ofcourse
      only works if you have the correct permissions. ;)

    * delete
      Deletes a modulefiles. As with makecache, this only works
      if you have the correct permissions.

    * autoload append|prepend|remove|list|purge [module name(s)]
      Manages the autoloading of modules when opening a new terminal.
";

fn is_shell_supported(shell: &str) -> bool {

    let mut shell_list = Vec::new();

    shell_list.push("tcsh");
    shell_list.push("csh");
    shell_list.push("bash");
    shell_list.push("zsh");
    // when noshell is selected, all output is printed
    // to stdout instead of the temp file
    // noshell is also useful for debugging purposes
    shell_list.push("noshell");
    shell_list.push("python");
    shell_list.push("perl");

    if shell_list.contains(&shell) {
        return true;
    }

    return false;
}

#[cfg(debug_assertions)]
fn release_debug() -> String {
    return String::from(" (debug)");
}
#[cfg(not(debug_assertions))]
fn release_debug() -> String {
    return String::from("");
}

fn usage(in_eval: bool) {
    let error_msg: &str;

    println_stderr!("  RSModules {}{} - {}", VERSION, release_debug(), AUTHORS);
    println_stderr!("");
    println_stderr!("  2017 - Ghent University / VIB");
    println_stderr!("  http://www.psb.ugent.be - http://www.ugent.be - http://www.vib.be");
    println_stderr!("");
    println_stderr!("");

    if in_eval {
        error_msg = "  Usage: module <load|unload|list|switch|purge|refresh|available|undo|info|makecache|delete|autoload> \
                     [module name]";
    } else {
        error_msg = "  Usage: rsmodules <shell> \
                     <load|unload|list|switch|purge|refresh|available|undo|info|makecache|delete|autoload> [module name]";
    }

    println_stderr!("{}", &error_msg);
    if !in_eval {
        println_stderr!("  Supported shells: bash, zsh, csh, tcsh, python, perl and \
                         noshell");
        println_stderr!("");
        println_stderr!("  When noshell is selected all output is printed to stdout,");
        println_stderr!("  module available will then print a nice list without gaps, which is");
        println_stderr!("  makes your life easier when you want to parse this output.");
    }
    println_stderr!("{}", &LONG_HELP);
}

fn set_global_tmpfile(tmp_file_path: String) {
    let mut tmp = TMPFILE_PATH.lock().unwrap();
    *tmp = tmp_file_path;

    let mut tmp = TMPFILE_INITIALIZED.lock().unwrap();
    *tmp = true;
}

fn run(args: &Vec<String>) {
    let command: &str;
    let tmp: String;
    let mut modulename: &str = "";

    let (shell, shell_width) = rsmod::get_shell_info();

    ////

    if !is_shell_supported(&shell) {
        usage(false);
        rsmod::crash(CRASH_UNSUPPORTED_SHELL,
                     &format!("{} is not a supported shell", shell));
    }

    let modulepaths = rsmod::get_module_paths(false);

    // create temporary file in the home folder
    // if the file cannot be created try to create it
    // in /tmp, if that fails, the program exits
    //
    // ~/.rsmodulestmpXXXXXXXX
    // /tmp/.rsmodulestmpXXXXXXXX

    let mut tmpfile: File;

    let rstr: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(8)
        .collect();

    let mut tmp_file_path: PathBuf;


    match env::home_dir() {
        Some(path) => tmp_file_path = path,
        None => {
            show_warning!("We were unable to find your home directory, checking if /tmp is an \
                            option");

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
                    rsmod::crash(CRASH_FAILED_TO_CREATE_TEMPORARY_FILE,
                                 &format!("Failed to create temporary file: {}", e));
                    return;
                }
            };
        }
    };

    if args.len() >= 3 {
        command = &args[2];
        let matches: bool;
        let mut modulenames: Vec<&str> = Vec::new();
        if args.len() > 3 {
            for i in 3..args.len() {
                modulenames.push(&args[i]);
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
        command_list.push("refresh");
        command_list.push("info");
        command_list.push("display");
        command_list.push("show");
        command_list.push("switch");
        command_list.push("makecache");
        command_list.push("help");
        command_list.push("undo");
        command_list.push("autoload");
        command_list.push("delete");
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
            usage(true);
            return;
        }

        let mut num_hits: i32 = 0;
        let mut command_hit: &str = "";

        for cmd in command_list {
            if cmd.starts_with(command) {
                num_hits += 1;
                command_hit = cmd;
            }
        }

        if num_hits != 1 {
            usage(true);
            return;
        } else {
            matches = true;

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
                let data = setenv("RSMODULES_UNDO".to_string(),
                                  format!("{} {}", command_hit, modulename.to_string()),
                                  &shell);
                crash_if_err!(CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                              tmpfile.write_all(data.as_bytes()));
            }

            if command_hit == "switch" {
                if args.len() != 5 {
                    usage(true);
                    return;
                }
            }

            if command_hit == "switch" {
                modulenames.reverse();
                let data = setenv("RSMODULES_UNDO".to_string(),
                                  format!("{} {}", command_hit, modulenames.join(" ")),
                                  &shell);
                crash_if_err!(CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                              tmpfile.write_all(data.as_bytes()));
            }

            if command_hit == "purge" {
                let loaded_list = rsmod::get_loaded_list();
                let mut args: Vec<String> = Vec::new();
                for (arg, _) in loaded_list.into_iter() {
                    args.push(arg);
                }
                let loadedmodules = args.join(" ");
                let data = setenv("RSMODULES_UNDO".to_string(),
                                  format!("unload {}", loadedmodules),
                                  &shell);
                crash_if_err!(CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                              tmpfile.write_all(data.as_bytes()));

            }

            let mut rsmod_command: Rsmodule = Rsmodule {
                cmd: &command_hit,
                typed_command: command,
                arg: modulename,
                search_path: &modulepaths,
                shell: &shell,
                shell_width: shell_width,
            };
            rsmod::command(&mut rsmod_command);

        }

        if !matches {
            usage(false);
        }
    }

    // when noshell is choosen, we just output to stdout
    // this is used for scripts that want to parse the module av output
    // for example for tab completion

    if shell != "noshell" && shell != "python" && shell != "perl" {
        // we want a self destructing tmpfile
        // so it must delete itself at the end of the run
        // if it crashes we still need to delete the file


        let cmd = format!("rm -f {}\n", tmp_file_path.display());

        let mut output_buffer = OUTPUT_BUFFER.lock().unwrap();
        let ref mut output_buffer = *output_buffer;
        output_buffer.push(cmd);

        for line in output_buffer {
            crash_if_err!(CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                          tmpfile.write_all(line.as_bytes()));
        }

        // source tmpfile
        println!("source {}", tmp_file_path.display());
    } else {
        remove_file(tmp_file_path.to_str().unwrap().to_string()).unwrap();
    }
}

pub fn setenv(var: String, val: String, shell: &str) -> String {
    let mut data: String = String::new();
    if shell == "bash" || shell == "zsh" {
        data = format!("export {}=\"{}\"\n", var, val);
    } else if shell == "tcsh" || shell == "csh" {
        data = format!("setenv {} \"{}\"\n", var, val);
    } else if shell == "python" {
        data = format!("os.environ[\"{}\"] = \"{}\";\n", var, val);
    } else if shell == "perl" {
        data = format!("$ENV{{{}}}=\"{}\";\n", var, val);
    }

    return data;
}

pub fn output(line: String) {
    let mut output_buffer = OUTPUT_BUFFER.lock().unwrap();
    let ref mut output_buffer = *output_buffer;
    output_buffer.push(line);
}

fn init() {
    let mut tmp = TMPFILE_INITIALIZED.lock().unwrap();
    *tmp = false;
}

fn main() {
    init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {

        if !wizard::run(false) {
            usage(false);
        }
        return;
    }

    if args.len() == 2 {
        usage(true);
    }

    // FIXME: args.get(1) == Some(&"-h")
    if args.len() >= 2 && (&args[1] == "-h" || &args[1] == "--help") {
        usage(false);
        return;
    } else if args.len() >= 3 && (&args[1] == "-h" || &args[1] == "--help") {
        usage(false);
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
    }
}
