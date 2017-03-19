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
    print!("{}: ", msg);
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

fn update_setup_rmodules_c_sh() {
    // no point in setting the env var, we are not running in the alias
    // env::set_var("MODULEPATH", path);
    // just update the setup_rmodules.(c)sh files and copy them to /etc/profile.d
    // if no permissions, tell them if they are an admin to run this as root
    // or just throw it in .bashrc and .personal_cshrc -> or first check if
    // /etc/profile.d/rmodules.csh link exists


    // println!("and now open a new terminal and type: module");
}

pub fn run() -> bool {
    let module_paths: Vec<String> = super::rmod::get_module_paths(true);

    if module_paths.len() == 0 {
        //            println!("No $MODULEPATH found, want to add one ? [Y/n]");
        let mut line = read_input("No $MODULEPATH found, want to add one ? [Y/n]");
        //          let stdin = io::stdin();
        //          stdin.lock().read_line(&mut line).expect("Could not read line");
        if is_yes(line) {
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

                    update_setup_rmodules_c_sh();
                    return true;
                } else {
                    return false;
                }

            } else if Path::new(path).is_file() {
                crash!(super::CRASH_MODULEPATH_IS_FILE,
                       "Modulepath cannot be a file");
            } else {
                if is_yes(read_input(format!("{} doesn't exist, do you want to create it ? \
                                              [Y/n]",
                                             path)
                    .as_ref())) {
                    //std::fs::create_dir_all(path);
                    create_dir_all(path).unwrap();
                    update_setup_rmodules_c_sh();
                    return true;
                } else {
                    return false;
                }
            }
        }
    }

    return false;
}
