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

use std::env;
use std::fs::create_dir_all;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::symlink;
use std::path::Path;
extern crate shellexpand;
use super::rsmod::crash;
use regex::Regex;

use users::get_current_uid;

fn read_input(msg: &str) -> String {
    read_input_shell(msg, "noshell")
}

pub fn read_input_shell(msg: &str, shell: &str) -> String {
    if shell == "noshell" {
        print!("{}", msg);
    } else {
        print_stderr!("{}", msg);
    }
    io::stdout().flush().unwrap();
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line).expect("Could not read line");
    line
}

pub fn is_yes(answer: &str) -> bool {
    if answer == "Y\n" || answer == "y\n" || answer == "\n" || answer == "yes\n" || answer == "Yes\n" || answer == "YES\n" {
        return true;
    }

    false
}

fn print_title(title: &str) {
    println!("    {}", title);
    println!("    {:=<1$}", "=", title.len());
    println!();
}

fn update_setup_rsmodules_c_sh(recursive: bool, path: &str) {
    // no point in setting the env var, we are not running in the alias
    // env::set_var("MODULEPATH", path);
    // just update the setup_rsmodules.(c)sh files and copy them to /etc/profile.d
    // if no permissions, tell them if they are an admin to run this as root
    // or just throw it in .bashrc and .personal_cshrc -> or first check if

    let executable_path = env::current_exe().unwrap();
    let executable_path = executable_path.parent();
    let current_path_sh: &str = &format!("{}/setup_rsmodules.sh", executable_path.unwrap().display());
    let current_path_csh: &str = &format!("{}/setup_rsmodules.csh", executable_path.unwrap().display());

    let bash_result: bool;
    let csh_result: bool;
    let bash_result2: bool;
    let csh_result2: bool;

    // update init files before we link them
    if !Path::new(current_path_sh).is_file() {
        crash!(
            super::CRASH_MISSING_INIT_FILES,
            "{} should be in the same folder as {}",
            current_path_sh,
            env::current_exe().unwrap().display()
        );
    } else {
        // add path to the file
        // use detect_line but with a regex: export MODULEPATH="(randomblah)"
        // and replace with export MODULEPATH="(randomblah):OURNEWPATH"
        bash_result = add_path(path, current_path_sh, "MODULEPATH", true);
        bash_result2 = add_path(
            &format!("{}", executable_path.unwrap().display()),
            current_path_sh,
            "RSMODULES_INSTALL_DIR",
            false,
        );
    }

    if !Path::new(current_path_csh).is_file() {
        crash!(
            super::CRASH_MISSING_INIT_FILES,
            "{} should be in the same folder as {}",
            current_path_csh,
            env::current_exe().unwrap().display()
        );
    } else {
        // add path to the file
        // use detect_line but with a regex: setenv MODULEPATH "(randomblah)"
        // and replace with setenv MODULEPATH "(randomblah):OURNEWPATH"
        csh_result = add_path(path, current_path_csh, "MODULEPATH", true);
        csh_result2 = add_path(
            &format!("{}", executable_path.unwrap().display()),
            current_path_csh,
            "RSMODULES_INSTALL_DIR",
            false,
        );
    }

    if (bash_result || bash_result2) && (csh_result || csh_result2) {
        println!();
        println!("    Successfully modified:");
    }

    if bash_result || bash_result2 {
        println!("    - {}", current_path_sh);
    }

    if csh_result || csh_result2 {
        println!("    - {}", current_path_csh);
    }

    if get_current_uid() == 0 {
        let path_sh: &str = "/etc/profile.d/rsmodules.sh";
        let path_csh: &str = "/etc/profile.d/rsmodules.csh";

        if !Path::new(path_sh).exists() || !Path::new(path_csh).exists() {
            println!();
            if !recursive {
                print_title("ENVIRONMENT SETUP");
            }
            if is_yes(&read_input(
                " * rsmodules is not setup yet to autoload when a user \
                 opens a terminal. Do you want to do this now ? [Y/n]: ",
            )) {
                let mut bash_success: bool = false;
                let mut csh_success: bool = false;
                println!();
                match symlink(current_path_sh, path_sh) {
                    Ok(_) => {
                        println!("    - Created symlink {} -> {}", current_path_sh, path_sh);
                        bash_success = true;
                    }
                    Err(msg) => println!("    - Could not create symlink {} -> {} ({})", current_path_sh, path_sh, msg),
                }

                match symlink(current_path_csh, path_csh) {
                    Ok(_) => {
                        println!("    - Created symlink {} => {}", current_path_csh, path_csh);
                        csh_success = true;
                    }
                    Err(msg) => println!(
                        "    - Could not create symlink {} => {} ({})",
                        current_path_csh, path_csh, msg
                    ),
                }

                if bash_success || csh_success {
                    println!("\n    On next login the command 'module' will be available.");
                    println!("    To have it active in the current terminal, type this:");
                    println!("    bash or zsh : source {}", current_path_sh);
                    println!("    csh or tcsh : source {}", current_path_csh);
                }
            }
        }
    } else {
        let path_sh: &str = &shellexpand::tilde("~/.rsmodules.sh");
        let path_csh: &str = &shellexpand::tilde("~/.rsmodules.csh");

        if !Path::new(path_sh).exists()
            || !Path::new(path_csh).exists()
            || !detect_line("source ~/.rsmodules.sh", &shellexpand::tilde("~/.bashrc"))
            || !detect_line("source ~/.rsmodules.csh", &shellexpand::tilde("~/.cshrc"))
        {
            println!();
            if !recursive {
                print_title("ENVIRONMENT SETUP");
            }
            if is_yes(&read_input(
                " * rsmodules is not setup yet to autoload when you \
                 open a new terminal.\n    Do you want to do this now ? [Y/n]: ",
            )) {
                // want to link rsmodules to /home and add it to bashrc
                // read .cshrc and .bashrc line by line
                // to detect if source ~/rsmodules.(c)sh exists in it
                // read filename line by line, and push it to modules

                println!();

                match symlink(current_path_sh, path_sh) {
                    Ok(_) => println!("    - Created symlink {} => {}", current_path_sh, path_sh),
                    Err(msg) => println!("    - Could not create symlink {} => {} ({})", current_path_sh, path_sh, msg),
                }

                match symlink(current_path_csh, path_csh) {
                    Ok(_) => println!("    - Created symlink {} => {}", current_path_csh, path_csh),
                    Err(msg) => println!(
                        "    - Could not create symlink {} => {} ({})",
                        current_path_csh, path_csh, msg
                    ),
                }

                let mut bash_updated: bool = true;
                let mut csh_updated: bool = true;

                let detected_sh: bool = detect_line("source ~/.rsmodules.sh", &shellexpand::tilde("~/.bashrc"));
                let detected_csh: bool = detect_line("source ~/.rsmodules.csh", &shellexpand::tilde("~/.cshrc"));

                if !detected_sh || !detected_csh {
                    println!();
                }

                if !detected_sh {
                    bash_updated = append_line("source ~/.rsmodules.sh", &shellexpand::tilde("~/.bashrc"), true, false);
                }

                if !detected_csh {
                    csh_updated = append_line("source ~/.rsmodules.csh", &shellexpand::tilde("~/.cshrc"), true, false);
                }

                if bash_updated || csh_updated {
                    println!("\n    On next login the command 'module' will be available.");
                    println!("\n    To have it active in the current terminal, type this:");
                }
                if bash_updated {
                    println!("    bash or zsh : source ~/.rsmodules.sh");
                }
                if csh_updated {
                    println!("    csh or tcsh : source ~/.rsmodules.csh");
                }
                println!();

                // create a dummy modules
                append_line(
                    "prepend_path(\"PATH\",\"~/bin\")",
                    &format!("{}/testmodule", path),
                    false,
                    false,
                );
                append_line(
                    "description(\"This is just a sample module, which adds ~/bin to your path\")",
                    &format!("{}/testmodule", path),
                    false,
                    false,
                );
                // tell them to run the module avail command
                println!("    Now run the command: module available");
            }
        }
    }
}

pub fn append_line(line: &str, filename: &str, verbose: bool, stderr: bool) -> bool {
    let mut file: File = match OpenOptions::new().write(true).append(true).create(true).open(filename) {
        Ok(fileresult) => fileresult,
        Err(e) => {
            if stderr {
                eprintln!("  Error: cannot append to file {} ({})", filename, e);
            } else {
                println!("    - Cannot append to file {} ({})", filename, e);
            }
            return false;
        }
    };

    if let Err(e) = writeln!(file, "{}", line) {
        crash(
            super::CRASH_CANNOT_ADD_TO_ENV,
            &format!("Cannot append to file {} ({})", filename, e),
        );
    }

    if verbose {
        if stderr {
            eprintln!("    Succesfully added '{}' to {}", line, filename);
        } else {
            println!("    - Succesfully added '{}' to {}", line, filename);
        }
    }

    true
}

pub fn detect_line(line: &str, file: &str) -> bool {
    if Path::new(file).is_file() {
        let file: File = match File::open(file) {
            Ok(file) => file,
            Err(_) => {
                return false;
            }
        };

        let file = BufReader::new(file);
        for (_, entry) in file.lines().enumerate() {
            let buffer = entry.unwrap();
            if buffer == line {
                return true;
            }
        }
    }

    false
}

// go over the file line by line, do we have
// a export MODULEPATH="" match, replace it
// same for setenv MODULEPATH ""
fn add_path(newpath: &str, filename: &str, variable: &str, append: bool) -> bool {
    let mut newbuffer: Vec<String> = Vec::new();

    if Path::new(filename).is_file() {
        let file: File = match File::open(filename) {
            Ok(file) => file,
            Err(_) => {
                return false;
            }
        };

        let file = BufReader::new(file);
        for (_, entry) in file.lines().enumerate() {
            let buffer = entry.unwrap();
            newbuffer.push(set_path(&buffer, newpath, variable, append));
        }
    }

    if !newbuffer.is_empty() {
        let mut file: File = match OpenOptions::new().write(true).open(filename) {
            Ok(fileresult) => fileresult,
            Err(e) => {
                println!("    - Cannot write to file {} ({})", filename, e);
                return false;
            }
        };

        for newline in newbuffer {
            if let Err(e) = writeln!(file, "{}", newline) {
                crash(
                    super::CRASH_CANNOT_ADD_TO_ENV,
                    &format!("Cannot write to file {} ({})", filename, e),
                );
            }
        }
    }

    true
}

// match against export MODULEPATH="" and setenv MODULEPATH ""
// and add the new path to it
fn set_path(input: &str, path: &str, variable: &str, append: bool) -> String {
    let re = Regex::new(&format!(
        r#"^\s*(?P<export>export|setenv)\s+{}(?P<equals>[= ]?)"(?P<value>.*)""#,
        variable
    ))
    .unwrap();

    let mut output: String = input.to_string();
    for cap in re.captures_iter(input) {
        let value = &cap["value"];
        let value: Vec<&str> = value.split(':').collect();

        for existing_path in &value {
            if existing_path == &path {
                return String::from(input);
            }
        }

        if append {
            if !(value.len() == 1 && value[0] == "" || value.is_empty()) {
                output = format!(
                    r#"{} {}{}"{}:{}""#,
                    &cap["export"], variable, &cap["equals"], &cap["value"], path
                );
            } else {
                output = format!(r#"{} {}{}"{}""#, &cap["export"], variable, &cap["equals"], path);
            }
        } else {
            output = format!(r#"{} {}{}"{}""#, &cap["export"], variable, &cap["equals"], path);
        }
    }

    output
}

// if no modulepath variable found, or it is empty
// start a wizard to add one to the path
// if uid = 0 suggest /usr/local/modulefiles as
// module path
// else : suggest ~/modulefiles as module path
// once a path is created and added to the
// $MODULEPATH envvar, start wizard to
// create a modulefile
// also update the setup_rsmodules.(c)sh files
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
    let module_paths: Vec<String> = super::rsmod::get_module_paths(true);

    if module_paths.is_empty() {
        // TODO: ask if we have to copy rsmodules to a different folder
        // before we continue
        // "it looks like rsmodules isn't setup yet, blabla, do you want to"

        println!();

        let mut line = if !recursive {
            print_title("MODULEPATH configuration");
            read_input(" * No $MODULEPATH found, want to add one ? [Y/n]:")
        } else {
            String::new()
        };

        if is_yes(&line) || recursive {
            let home_modules = &shellexpand::tilde("~/modules");
            let mut path = if get_current_uid() == 0 {
                "/usr/local/modules"
            } else {
                home_modules
            };
            line = read_input(
                format!(
                    " * Please enter a path where you want to save your module \
                     files [{}]: ",
                    path
                )
                .as_ref(),
            );

            if line != "\n" {
                /*
                let len = line.len();
                line.truncate(len - 1);
                path = line.as_ref();
                */
                path = line.trim_right_matches('\n');
            }

            if Path::new(path).is_dir() {
                if is_yes(&read_input(
                    " * Path already exists, are you sure you want to continue ? \
                     [Y/n]: ",
                )) {
                    update_setup_rsmodules_c_sh(false, path);
                    return true;
                } else {
                    return run(true);
                }
            } else if Path::new(path).is_file() {
                crash(super::CRASH_MODULEPATH_IS_FILE, "Modulepath cannot be a file");
                return false;
            } else if is_yes(&read_input(
                format!(
                    " * The folder {} doesn't exist, do you want to \
                     create it ? [Y/n]: ",
                    path
                )
                .as_ref(),
            )) {
                create_dir_all(path).unwrap();
                println!();
                println!("    - Succesfully created {}", path);
                update_setup_rsmodules_c_sh(false, path);
                return true;
            } else {
                println!();
                println!("   ==== WARNING: Don't forget to create: {} ====", path);
                update_setup_rsmodules_c_sh(false, path);
                return true;
            }
        }
    }

    false
}
