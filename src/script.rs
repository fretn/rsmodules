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
extern crate rhai;
use self::rhai::{Engine, FnRegister};
use std::env;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::{Path, PathBuf, is_separator};
use std::io::Write;
use std::fs::{metadata, read_dir};
use std::os::unix::fs::PermissionsExt;
use super::{Rsmodule, get_shell_info, echo};

// WARNING: the scripts don't support tabbed indents in if else structures

lazy_static! {
    static ref ENV_VARS: Mutex<Vec<(String, String)>> = Mutex::new(vec![]);
    static ref COMMANDS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref CONFLICT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref INFO_DESCRIPTION: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_GENERAL: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_LD_LIBRARY_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PYTHONPATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PERL5LIB: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref LOAD: Mutex<Vec<String>> = Mutex::new(vec![]);
}

fn init_vars_and_commands() {
    ENV_VARS.lock().unwrap().clear();
    COMMANDS.lock().unwrap().clear();
    INFO_DESCRIPTION.lock().unwrap().clear();
    INFO_GENERAL.lock().unwrap().clear();
    INFO_PATH.lock().unwrap().clear();
    INFO_LD_LIBRARY_PATH.lock().unwrap().clear();
    INFO_PYTHONPATH.lock().unwrap().clear();
    INFO_PERL5LIB.lock().unwrap().clear();
    LOAD.lock().unwrap().clear();

    CONFLICT.store(false, Ordering::Relaxed);
}

fn add_to_env_vars(variable: &str, value: &str) {
    ENV_VARS.lock().unwrap().push((variable.to_string(), value.to_string()));
}

fn add_to_commands(data: &str) {
    COMMANDS.lock().unwrap().push(data.to_string());
}

fn add_to_info_general(data: &str) {
    INFO_GENERAL.lock().unwrap().push(data.to_string());
}

fn add_to_info_path(data: &str) {
    INFO_PATH.lock().unwrap().push(data.to_string());
}

fn add_to_info_ld_library_path(data: &str) {
    INFO_LD_LIBRARY_PATH.lock().unwrap().push(data.to_string());
}

fn add_to_info_pythonpath(data: &str) {
    INFO_PYTHONPATH.lock().unwrap().push(data.to_string());
}

fn add_to_info_perl5lib(data: &str) {
    INFO_PERL5LIB.lock().unwrap().push(data.to_string());
}

fn add_to_load(data: String) {
    LOAD.lock().unwrap().push(data);
}

// functions for load and unload
fn getenv(var: &str) -> String {

    match env::var(&var) {
        Ok(res) => res,
        Err(_) => {
            show_warning!("${} not found", var);
            String::from("")
        }
    }
}

// dummy functions for unloading
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn unsetenv_dummy(var: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn remove_path_dummy(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn system_dummy(cmd: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn load_dummy(module: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn conflict_dummy(module: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn unload_dummy(module: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn description_dummy(desc: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn getenv_dummy(var: String) -> String {
    String::new()
}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn prepend_path_dummy(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn append_path_dummy(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn setenv_dummy(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn set_alias_dummy(name: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn is_loaded_dummy(var: String) -> bool {
    true
}

// unload functions

#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn setenv_unload(var: String, val: String) {
    unsetenv(var);
}

// info functions

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn setenv_info(var: String, val: String) {
    add_to_info_general(&format!("{}={}", var, val));
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn prepend_path_info(var: String, val: String) {
    if var == "PATH" {
        add_to_info_path(&val);
    } else if var == "LD_LIBRARY_PATH" {
        add_to_info_ld_library_path(&val);
    } else if var == "PYTHONPATH" {
        add_to_info_pythonpath(&val);
    } else if var == "PERL5LIB" {
        add_to_info_perl5lib(&val);
    } else {
        add_to_info_general(&format!("{}={}", var, val));
    }
}
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn append_path_info(var: String, val: String) {
    if var == "PATH" {
        add_to_info_path(&val);
    } else if var == "LD_LIBRARY_PATH" {
        add_to_info_ld_library_path(&val);
    } else if var == "PYTHONPATH" {
        add_to_info_pythonpath(&val);
    } else if var == "PERL5LIB" {
        add_to_info_perl5lib(&val);
    } else {
        add_to_info_general(&format!("{}={}", var, val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn load_info(module: String) {
    add_to_load(module);
}
// load functions

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn setenv(var: String, val: String) {
    add_to_env_vars(&var, &val);
    env::set_var(&var, val);
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn is_loaded(var: String) -> bool {
    super::is_module_loaded(&var, false)
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn unsetenv(var: String) {
    let (shell, _) = get_shell_info();
    if shell == "bash" || shell == "zsh" {
        add_to_commands(&format!("unset \"{}\"", var));
    } else if shell == "perl" {
        add_to_commands(&format!("undef \"{}\"", var));
    } else if shell == "python" {
        add_to_commands(&format!("os.environ[\"{}\"] = \"\";", var));
        add_to_commands(&format!("del os.environ[\"{}\"];", var));
    } else {
        add_to_commands(&format!("unsetenv \"{}\"", var));
    }
    env::remove_var(&var);
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn prepend_path(var: String, val: String) {
    let mut current_val: String = String::from("");
    let mut notfound: bool = false;

    match env::var(&var) {
        Ok(res) => current_val = res,
        Err(_) => {
            //show_warning!("${} not found", var);
            notfound = true;
        }
    };

    if notfound {
        setenv(var, val);
    } else {
        let final_val = format!("{}:{}", val, current_val);
        add_to_env_vars(&var, &final_val);
        env::set_var(&var, format!("{}:{}", val, current_val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn append_path(var: String, val: String) {
    let mut current_val: String = String::from("");
    let mut notfound: bool = false;

    match env::var(&var) {
        Ok(res) => current_val = res,
        Err(_) => {
            //show_warning!("${} not found", var);
            notfound = true;
        }
    };

    if notfound {
        setenv(var, val);
    } else {
        let final_val = format!("{}:{}", current_val, val);
        add_to_env_vars(&var, &final_val);
        env::set_var(&var, format!("{}:{}", current_val, val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn remove_path(var: String, val: String) {
    let current_val: String;

    match env::var(&var) {
        Ok(res) => current_val = res,
        Err(_) => {
            //show_warning!("${} not found", var);
            return;
        }
    };

    let mut values: Vec<&str> = current_val.split(':').collect();
    values.retain(|&x| x != val);

    let result = values.join(":");

    add_to_env_vars(&var, &result);
    env::set_var(&var, result);
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn unset_alias(name: String, val: String) {
    let (shell, _) = get_shell_info();
    if shell == "bash" || shell == "zsh" {
        add_to_commands(&format!("unalias \"{}\"", name));
    } else if shell == "tcsh" || shell == "csh" {
        add_to_commands(&format!("unalias \"{}={}\"", name, val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn set_alias(name: String, val: String) {
    let (shell, _) = get_shell_info();
    if shell != "python" && shell != "perl" {
        add_to_commands(&format!("alias {}=\"{}\"", name, val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn system(cmd: String) {
    let (shell, _) = get_shell_info();
    if shell != "python" && shell != "perl" {
        add_to_commands(&cmd);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn load(module: String) {
    let (shell, _) = get_shell_info();

    let modulepaths = super::get_module_paths(false);
    let mut rsmod_command: Rsmodule = Rsmodule {
        cmd: "load",
        typed_command: "load",
        arg: &module,
        search_path: &modulepaths,
        shell: &shell,
        shell_width: 80,
    };
    super::command(&mut rsmod_command);
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn conflict(module: String) {

    if super::is_module_loaded(module.as_ref(), false) {
        let (shell, _) = get_shell_info();

        let mut spaces = "  ";

        let (mut bold_start, mut bold_end) = if shell == "tcsh" || shell == "csh" {
            ("\\033[1m", "\\033[0m")
        } else {
            ("$(tput bold)", "$(tput sgr0)")
        };

        if shell == "noshell" || shell == "perl" || shell == "python" {
            spaces = "";
            bold_start = "";
            bold_end = "";
        }

        let shell = &shell;
        if shell != "noshell" {
            echo("", shell);
        }
        echo(&format!("{}Cannot continue because the module {}{}{} is loaded.",
                      spaces,
                      bold_start,
                      module,
                      bold_end),
             shell);

        if shell != "noshell" {
            echo(&format!("{}You'll need to unload {}{}{} before you can continue:",
                          spaces,
                          bold_start,
                          module,
                          bold_end),
                 shell);
            echo("", shell);
            echo(&format!("{}{}module unload {}{}",
                          bold_start,
                          spaces,
                          module,
                          bold_end),
                 shell);
            echo("", shell);
        }
        CONFLICT.store(true, Ordering::Relaxed);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn unload(module: String) {
    let (shell, _) = get_shell_info();
    let shell: &String = &shell;
    let modulepaths = super::get_module_paths(false);
    let mut rsmod_command: Rsmodule = Rsmodule {
        cmd: "unload",
        typed_command: "unload",
        arg: &module,
        search_path: &modulepaths,
        shell: shell,
        shell_width: 80,
    };
    super::command(&mut rsmod_command);
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn description(desc: String) {
    INFO_DESCRIPTION.lock().unwrap().push(desc.replace("\"", "\\\""));
}

fn description_cache(desc: &str) {
    add_to_info_general(desc);
}

pub fn run(path: &PathBuf, action: &str) {
    let mut engine = Engine::new();

    init_vars_and_commands();

    if action == "unload" {
        // for unloading, we swap some functions
        // prepand_path and append_path are just remove_path
        // setenv should be an alternative to unsetenv
        // the others arent used
        engine.register_fn("setenv", setenv_unload);
        engine.register_fn("unsetenv", unsetenv_dummy);
        engine.register_fn("prepend_path", remove_path);
        engine.register_fn("append_path", remove_path);
        engine.register_fn("remove_path", remove_path_dummy);
        engine.register_fn("system", system_dummy);
        engine.register_fn("load", load_dummy);
        engine.register_fn("conflict", conflict_dummy);
        engine.register_fn("unload", unload_dummy);
        engine.register_fn("getenv", getenv_dummy); // need getenv_dummy instead ??
        engine.register_fn("description", description_dummy);
        engine.register_fn("set_alias", unset_alias);
        engine.register_fn("is_loaded", is_loaded_dummy);

    } else if action == "load" {
        engine.register_fn("setenv", setenv);
        engine.register_fn("unsetenv", unsetenv);
        engine.register_fn("prepend_path", prepend_path);
        engine.register_fn("append_path", append_path);
        engine.register_fn("remove_path", remove_path);
        engine.register_fn("system", system);
        engine.register_fn("load", load);
        engine.register_fn("conflict", conflict);
        engine.register_fn("unload", unload);
        engine.register_fn("getenv", getenv);
        engine.register_fn("description", description_dummy);
        engine.register_fn("set_alias", set_alias);
        engine.register_fn("is_loaded", is_loaded);

    } else if action == "info" {
        engine.register_fn("setenv", setenv_info);
        engine.register_fn("unsetenv", unsetenv_dummy);
        engine.register_fn("prepend_path", prepend_path_info);
        engine.register_fn("append_path", append_path_info);
        engine.register_fn("remove_path", remove_path_dummy);
        engine.register_fn("system", system_dummy);
        engine.register_fn("load", load_info);
        engine.register_fn("conflict", conflict_dummy);
        engine.register_fn("unload", unload_dummy);
        engine.register_fn("getenv", getenv_dummy);
        engine.register_fn("description", description);
        engine.register_fn("set_alias", set_alias_dummy);
        engine.register_fn("is_loaded", is_loaded);

    } else if action == "description" {
        engine.register_fn("setenv", setenv_dummy);
        engine.register_fn("unsetenv", unsetenv_dummy);
        engine.register_fn("prepend_path", prepend_path_dummy);
        engine.register_fn("append_path", append_path_dummy);
        engine.register_fn("remove_path", remove_path_dummy);
        engine.register_fn("system", system_dummy);
        engine.register_fn("load", load_dummy);
        engine.register_fn("conflict", conflict_dummy);
        engine.register_fn("unload", unload_dummy);
        engine.register_fn("getenv", getenv_dummy);
        engine.register_fn("description", description_cache);
        engine.register_fn("set_alias", set_alias);
        engine.register_fn("is_loaded", is_loaded);

    }

    match engine.eval_file::<String>(path.to_string_lossy().into_owned().as_ref()) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            if e.to_string() != "Cast of output failed" {
                show_warning!("modulescript error: {} ({})",
                              e.to_string(),
                              path.to_string_lossy().into_owned());
            }
        }
    }
}

pub fn get_description() -> Vec<String> {

    let mut output: Vec<String> = Vec::new();


    // there can be multiple description calls, but
    // only store the first line of the description in
    // the cache file
    output.push(INFO_GENERAL.lock().unwrap().get(0).unwrap().to_string());

    output
}

pub fn get_output(selected_module: &str, action: &str, shell: &str) -> Vec<String> {

    if CONFLICT.load(Ordering::Relaxed) {
        return Vec::new();
    }

    if action == "unload" {
        remove_path(super::ENV_LOADEDMODULES.to_string(),
                    selected_module.to_string());
    } else if action == "load" {
        prepend_path(super::ENV_LOADEDMODULES.to_string(),
                     selected_module.to_string());
    }

    // this part must be below the above part
    let mut output: Vec<String> = Vec::new();

    for result in ENV_VARS.lock().unwrap().iter() {
        let mut data: String = String::new();
        if shell == "bash" || shell == "zsh" {
            data = format!("export {}=\"{}\"", result.0, result.1);
        } else if shell == "tcsh" || shell == "csh" {
            data = format!("setenv {} \"{}\"", result.0, result.1);
        } else if shell == "python" {
            data = format!("os.environ[\"{}\"] = \"{}\";", result.0, result.1);
        } else if shell == "perl" {
            data = format!("$ENV{{{}}}=\"{}\";", result.0, result.1);
        }

        if shell != "noshell" {
            output.push(data);
        }
    }

    for line in COMMANDS.lock().unwrap().iter() {
        output.push(line.to_string());
    }

    output
}

// this function prints information about the module
pub fn get_info(shell: &str, module: &str) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut got_output: bool = false;

    let (bold_start, bold_end) = if shell == "tcsh" || shell == "csh" {
        ("\\033[1m", "\\033[0m")
    } else {
        ("$(tput bold)", "$(tput sgr0)")
    };

    //output.push(format!("echo \"{:=^1$}\"", module.to_string(), module.len()+5));
    output.push(format!("echo \"{}{}{}\"",
                        bold_start,
                        "=".repeat(module.len() + 4),
                        bold_end));
    output.push(format!("echo \"{}= {} ={}\"",
                        bold_start,
                        module.to_string(),
                        bold_end));
    output.push(format!("echo \"{}{}{}\"",
                        bold_start,
                        "=".repeat(module.len() + 4),
                        bold_end));
    output.push(String::from("echo \"\""));

    if INFO_DESCRIPTION.lock().unwrap().iter().len() > 0 {
        got_output = true;
    }
    for line in INFO_DESCRIPTION.lock().unwrap().iter() {
        if shell == "bash" || shell == "zsh" {
            output.push(format!("echo $\"{}\"", line.to_string()));
        } else if shell == "csh" || shell == "tcsh" {
            output.push(format!("echo \"{}\"", line.to_string().replace("\n", "\\n")));
        } else {
            output.push(format!("echo \"{}\"", line.to_string()));
        }
    }

    if INFO_GENERAL.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}Sets the following variables: {}\"",
                            bold_start,
                            bold_end));
        got_output = true;
    }
    for line in INFO_GENERAL.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    // TODO: find man pages and let the user know

    if INFO_PATH.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}Executables can be found in: {}\"",
                            bold_start,
                            bold_end));
        got_output = true;
    }
    for line in INFO_PATH.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if INFO_LD_LIBRARY_PATH.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}Libraries can be found in: {}\"",
                            bold_start,
                            bold_end));
        got_output = true;
    }
    for line in INFO_LD_LIBRARY_PATH.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if INFO_PYTHONPATH.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}\\$PYTHONPATH: {}\"", bold_start, bold_end));
        got_output = true;
    }
    for line in INFO_PYTHONPATH.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if INFO_PERL5LIB.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}\\$PERL5LIB: {}\"", bold_start, bold_end));
        got_output = true;
    }
    for line in INFO_PERL5LIB.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if LOAD.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}Depends on: {}\"", bold_start, bold_end));
        got_output = true;
    }
    for line in LOAD.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    let mut execs: Vec<String> = Vec::new();
    for line in INFO_PATH.lock().unwrap().iter() {

        if Path::new(line).is_dir() {
            let entries = match read_dir(line) {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            for entry in entries {

                let path = match entry {
                    Ok(p) => p.path(),
                    Err(_) => continue,
                };

                if path.is_dir() {
                    continue;
                }

                if is_executable_file(&path) {
                    execs.push(format!("echo '{}'", strip_dir(path.to_str().unwrap())));
                    got_output = true;
                }
            }
        }
    }

    if !execs.is_empty() {
        output.push(String::from("echo ''"));
        if execs.len() > 1 {
            output.push(format!("echo \"{}Try one of these commands to run the program: {}\"",
                                bold_start,
                                bold_end));
        } else {
            output.push(format!("echo \"{}Try this command to run the program: {}\"",
                                bold_start,
                                bold_end));
        }
    }

    execs.sort();
    for exec in execs {
        output.push(exec);
    }

    if got_output {
        output.push(String::from("echo ''"));
    }


    output
}

// thx uucore
fn strip_dir(fullname: &str) -> String {
    // Remove all platform-specific path separators from the end
    let mut path: String = fullname.chars().rev().skip_while(|&ch| is_separator(ch)).collect();

    // Undo reverse
    path = path.chars().rev().collect();

    // Convert to path buffer and get last path component
    let pb = PathBuf::from(path);
    match pb.components().last() {
        Some(c) => c.as_os_str().to_str().unwrap().to_owned(),
        None => "".to_owned(),
    }
}

fn is_executable_file(path: &PathBuf) -> bool {
    let meta = match metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    let perm = meta.permissions();

    let mut octal_number = 0;
    let mut decimal_number = perm.mode();
    let mut i: u32 = 1;
    while decimal_number != 0 {
        octal_number += (decimal_number % 8) * i;
        decimal_number /= 8;
        i *= 10;
    }

    let perm: String = octal_number.to_string();
    let perm = perm.split("");
    let mut counter = 0;
    for p in perm {
        if p != "" {
            let mut perms = p.parse::<i32>().unwrap();

            if perms - 4 >= 0 {
                perms -= 4;
            }

            if perms - 2 >= 0 {
                perms -= 2;
            }

            if perms - 1 >= 0 && counter == 2 {
                return true;
            }
            counter += 1;

        }
    }

    false
}
