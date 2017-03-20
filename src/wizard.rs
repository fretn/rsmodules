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

use std::io::{self, Write, BufRead};
use std::fs::create_dir_all;
use std::path::Path;
extern crate shellexpand;


use users::get_current_uid;

fn read_input(msg: &str) -> String {
    print!("  * {}: ", msg);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line).expect("Could not read line");
    return line;
}

fn is_yes(answer: String) -> bool {

    if answer == "Y\n" || answer == "y\n" || answer == "\n" || answer == "yes\n" ||
       answer == "Yes\n" || answer == "YES\n" {
        return true;
    }

    return false;
}

fn print_title(title: &str) {
    println!("    {}", title);
    println!("    {:=<1$}", "=", title.len());
    println!("");
}

fn update_setup_rmodules_c_sh(recursive: bool) {
    // no point in setting the env var, we are not running in the alias
    // env::set_var("MODULEPATH", path);
    // just update the setup_rmodules.(c)sh files and copy them to /etc/profile.d
    // if no permissions, tell them if they are an admin to run this as root
    // or just throw it in .bashrc and .personal_cshrc -> or first check if
    // /etc/profile.d/rmodules.csh link exists


    // do we create the rmodules.(c)sh files from code ?

    if get_current_uid() == 0 {
        if !Path::new("/etc/profile.d/rmodules.sh").exists() ||
           !Path::new("/etc/profile.d/rmodules.csh").exists() {
            println!("");
            if !recursive {
                print_title("ENVIRONMENT SETUP");
            }
            if is_yes(read_input("rmodules is not setup yet to autoload when a user \
                                opens a terminal.\n    Do you want to do this now ? [Y/n]")) {}

        }
    } else {
        let path_sh: &str = &shellexpand::tilde("~/rmodules.sh");
        let path_csh: &str = &shellexpand::tilde("~/rmodules.csh");
        if !Path::new(path_sh).exists() || !Path::new(path_csh).exists() {
            println!("");
            if !recursive {
                print_title("ENVIRONMENT SETUP");
            }
            // want to link rmodules to /home and add it to bashrc

        }
    }


    // println!("and now open a new terminal and type: module");
}

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

pub fn run(recursive: bool) -> bool {
    let module_paths: Vec<String> = super::rmod::get_module_paths(true);

    if module_paths.len() == 0 {


        // TODO: ask if we have to copy rmodules to a different folder
        // before we continue
        // "it looks like rmodules isn't setup yet, blabla, do you want to"


        println!("");

        let mut line: String = String::new();

        if !recursive {
            print_title("MODULEPATH configuration");
            line = read_input("No $MODULEPATH found, want to add one ? [Y/n]");
        }

        if is_yes(line) || recursive {
            let mut path: &str = &shellexpand::tilde("~/modules");
            if get_current_uid() == 0 {
                path = "/usr/local/modules";
            }
            line = read_input(format!("Please enter a path where you want to save your module \
                                       files [{}]",
                                      path)
                .as_ref());

            if line != "\n" {
                path = line.as_ref();
            }

            if Path::new(path).is_dir() {
                if is_yes(read_input("Path already exists, are you sure you want to continue ? \
                                      [Y/n]")) {

                    update_setup_rmodules_c_sh(false);
                    return true;
                } else {
                    return run(true);
                }

            } else if Path::new(path).is_file() {
                super::rmod::crash(super::CRASH_MODULEPATH_IS_FILE,
                                   "Modulepath cannot be a file");
                return false;
            } else {
                if is_yes(read_input(format!("The folder {} doesn't exist, do you want to \
                                              create it ? [Y/n]",
                                             path)
                    .as_ref())) {

                    create_dir_all(path).unwrap();
                    update_setup_rmodules_c_sh(false);

                    return true;
                } else {
                    println!("");
                    println!("   ==== WARNING: Don't forget to create: {} ====", path);
                    update_setup_rmodules_c_sh(false);
                    return true;
                }
            }
        }
    }

    return false;
}
