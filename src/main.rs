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

#[path = "rmodules.rs"]
mod rmod;

#[path = "wizard.rs"]
mod wizard;

use rmod::Rmodule;

extern crate rustc_serialize;
extern crate bincode;
extern crate walkdir;
extern crate users;

extern crate shellexpand;

use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::str::FromStr;

static CRASH_UNSUPPORTED_SHELL: i32 = 1;
static CRASH_FAILED_TO_CREATE_TEMPORARY_FILE: i32 = 2;
static CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE: i32 = 3;
static CRASH_NO_CACHE_FILES_FOUND: i32 = 4;
static CRASH_MODULE_NOT_FOUND: i32 = 5;
static CRASH_COULDNT_OPEN_CACHE_FILE: i32 = 5;
static CRASH_NO_ARGS: i32 = 6;
static CRASH_MODULEPATH_IS_FILE: i32 = 7;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

static LONG_HELP: &'static str = "

  rmodules manage your user environment on linux, macos, ...
  The rmodules package is a tool to help users modifying their environment
  during a session by using modulefiles.
  A modulefile contains all the settings needed to configure the shell for
  using a certain application.

  A modulefile sets or alters environment variables such as PATH,
  LD_LIBRARY_PATH, MANPATH, PYTHONPATH, PERL5LIB, ...

  Modulefiles can be shared by many users or can be used by individuals
  by setting up paths in the MODULEPATH environment variable. Once
  a modulepath is added the cache needs to be updated by invoking
  module makecache

  Modulefiles can be loaded and unloaded by the user whenever the
  module command is available.

  * module [subcommand] <module name>

    subcommands
    -----------

    * load [(partial) module name]
    * unload [(partial) module name]

      A partial module name is the part of the modulename
      before a slash, eg: you have module name 'rmodules/2.0.0'
      the partial name is 'rmodules'

    * list
      Lists all the loaded modules

    * purge
      Unloads all loaded modules

    * available <search string>
      Lists all the available module.
      If a <search string> is given then all modules which match
      the search string will be listed

    * info [(partial) module name]
      Gives more info about a module. Description, which
      variables it modifies and/or which commands are executed
      upon launch.

    * makecache
      Updates the .modulesindex file in all the paths that
      are found in the $MODULEPATHS variable. This ofcourse
      only works if you have the correct permissions ;)
";

fn is_shell_supported(shell: &str) -> bool {

    let mut shell_list = Vec::new();

    shell_list.push("tcsh");
    shell_list.push("csh");
    shell_list.push("bash");
    shell_list.push("zsh");

    if shell_list.contains(&shell) {
        return true;
    }

    return false;
}

fn usage(in_eval: bool) {
    let error_msg: &str;

    println_stderr!("  rmodules {} - {}", VERSION, AUTHORS);
    println_stderr!("");
    println_stderr!("  2017 - Ghent University / VIB");
    println_stderr!("  http://www.psb.ugent.be - http://www.ugent.be - http://www.vib.be");
    println_stderr!("");
    println_stderr!("");

    if in_eval {
        error_msg = "  Usage: module <load|unload|list|purge|available|info|makecache> [module \
                           name]";
    } else {
        error_msg = "  Usage: rmodules <shell> \
                     <load|unload|list|purge|available|info|makecache> [module name]";
    }

    println_stderr!("{}", &error_msg);
    println_stderr!("{}", &LONG_HELP);
}

fn run(args: &Vec<String>) {
    let mut shell: &str = &args[1];
    let command: &str;
    let mut modulename: &str = "";
    let mut shell_width: usize = 80;

    // the shell argument can either be 'bash', 'tcsh'
    // or the shellname comma shellwidth
    // bash,80 or csh,210 or bash,210 etc
    // if no width is specified, 80 is used as default width

    let shell_split: Vec<&str> = shell.split(',').collect();

    if shell_split.len() == 2 {
        shell_width = FromStr::from_str(shell_split[1]).unwrap();
        shell = shell_split[0];
    }

    ////

    if !is_shell_supported(shell) {
        usage(false);
        crash!(CRASH_UNSUPPORTED_SHELL,
               "{} is not a supported shell",
               shell);
    }

    // get install dir
    let mut install_dir: String = env::current_dir().unwrap().to_string_lossy().into_owned();

    match env::var("RMODULES_INSTALL_DIR") {
        Ok(path) => install_dir = path,
        Err(_) => {
            show_warning!("$RMODULES_INSTALL_DIR not found, using {}", install_dir);
        }
    };

    //let modules = rmod::get_module_list();
    let modulepaths = rmod::get_module_paths(false);

    // create temporary file in the home folder
    // if the file cannot be created try to create it
    // in /tmp, if that fails, the program exits
    //
    // ~/.rmodulestmpXXXXXXXX
    // /tmp/.rmodulestmpXXXXXXXX

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

    let filename: String = format!(".rmodulestmp{}", rstr);
    let filename: &str = filename.as_ref();
    tmp_file_path.push(filename);

    match File::create(&tmp_file_path) {
        Ok(file) => tmpfile = file,
        Err(_) => {
            // home exists but we can't create the temp file in it or
            // worst case, /tmp exists but we can't create the temp file in it
            tmp_file_path = env::temp_dir();
            let filename: String = format!(".rmodulestmp{}", rstr);
            let filename: &str = filename.as_ref();
            tmp_file_path.push(filename);

            match File::create(&tmp_file_path) {
                Ok(newfile) => tmpfile = newfile,
                Err(e) => {
                    crash!(CRASH_FAILED_TO_CREATE_TEMPORARY_FILE,
                           "Failed to create temporary file: {}",
                           e);
                    //return;
                }
            };
        }
    };

    if args.len() >= 3 {
        command = &args[2];
        let mut matches: bool = false;
        if args.len() > 3 {
            modulename = &args[3];
        }

        let mut command_list: Vec<&str> = Vec::new();
        command_list.push("load");
        command_list.push("unload");
        command_list.push("available");
        command_list.push("list");
        command_list.push("purge");
        command_list.push("info");
        command_list.push("makecache");
        command_list.push("help");
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

        for cmd in command_list {
            if command == "help" || command == "--help" || command == "-h" {
                usage(true);
                matches = true;
                break;
            }

            if cmd.starts_with(command) {
                let mut rmod_command: Rmodule = Rmodule {
                    cmd: cmd,
                    arg: modulename,
                    search_path: &modulepaths,
                    shell: shell,
                    shell_width: shell_width,
                    tmpfile: &tmpfile,
                    installdir: &install_dir,
                };
                rmod::command(&mut rmod_command);
                matches = true;
            }
        }

        if !matches {
            usage(false);
        }
    }

    // we want a self destructing tmpfile
    // so it must delete itself at the end of the run
    let cmd = format!("rm -f {}\n", tmp_file_path.display());
    crash_if_err!(CRASH_FAILED_TO_WRITE_TO_TEMPORARY_FILE,
                  tmpfile.write_all(cmd.as_bytes()));

    // source tmpfile
    println!("source {}", tmp_file_path.display());
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // if no modulepath variable found, or it is empty
        // start a wizard to add one to the path
        // if uid = 0 suggest /usr/local/modulefiles as
        // module path
        // else : suggest ~/modulefiles as module path
        // once a path is created and added to the
        // $MODULEPATH envvar, start wizard to
        // create a modulefile
        // also update the setup_rmodules.(c)sh files
        // and ask to put them in /etc/profile.d
        //
        // if modulepath found, but it is empty
        // start a wizard to add a module file
        // if .modulesindex doesn't exist
        // suggest the makecache command
        //
        // if modulepath found, and there are
        // module files but there is no .modulesindex file
        // suggest the makecache command
        //
        // else
        // crash with the help

        if !wizard::run() {
            crash!(CRASH_NO_ARGS,
                   "Try '{0} --help' for more information.",
                   executable!());
        }
    }

    if args.len() == 2 {
        // check if there are module files, or if there is a .modulesindex (see above)
        // else print usage
        usage(true);
    }

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
