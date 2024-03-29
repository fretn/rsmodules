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

use self::rhai::{Engine, RegisterFn};
use super::super::bold;
use super::{echo, get_shell_info, Rsmodule};
use is_executable::IsExecutable;
use regex::Regex;
use std::env;
use std::ffi::OsString;
use std::fs::read_dir;
use std::io::Write;
use std::path::{is_separator, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};

// WARNING: the scripts don't support tabbed indents in if else structures

#[derive(Debug, Clone)]
pub enum DeprecatedState {
    Not,
    Before,
    After,
}

#[derive(Debug, Clone)]
pub struct Deprecated {
    pub name: String,
    pub time: String,
    pub state: DeprecatedState,
}

impl Deprecated {
    pub fn new() -> Deprecated {
        Deprecated {
            name: String::new(),
            time: String::new(),
            state: DeprecatedState::Not,
        }
    }

    pub fn from(name: String, time: String, state: DeprecatedState) -> Deprecated {
        Deprecated {
            name: name,
            time: time,
            state: state,
        }
    }
}

lazy_static! {
    static ref ENV_VARS: Mutex<Vec<(String, String)>> = Mutex::new(vec![]);
    static ref COMMANDS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref CONFLICT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    //static ref DEPRECATED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    pub static ref DEPRECATED: Mutex<Deprecated> = Mutex::new(Deprecated::new());
    static ref README_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref README_MANPATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_DESCRIPTION: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_DEPRECATED: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_GENERAL: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref SOURCES: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_LD_LIBRARY_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PYTHONPATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PERL5LIB: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_BIN: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref LOAD: Mutex<Vec<String>> = Mutex::new(vec![]);
}

// lu! means lock().unwrap()
fn init_vars_and_commands() {
    lu!(ENV_VARS).clear();
    lu!(COMMANDS).clear();
    lu!(README_PATH).clear();
    lu!(README_MANPATH).clear();
    lu!(INFO_DESCRIPTION).clear();
    lu!(INFO_DEPRECATED).clear();
    lu!(INFO_GENERAL).clear();
    lu!(SOURCES).clear();
    lu!(INFO_PATH).clear();
    lu!(INFO_LD_LIBRARY_PATH).clear();
    lu!(INFO_PYTHONPATH).clear();
    lu!(INFO_PERL5LIB).clear();
    lu!(LOAD).clear();

    CONFLICT.store(false, Ordering::Relaxed);
    //DEPRECATED.store(false, Ordering::Relaxed);
    let mut deprecated = lu!(DEPRECATED);
    *deprecated = Deprecated::new();
}

fn add_to_env_vars(variable: &str, value: &str) {
    lu!(ENV_VARS).push((variable.to_string(), value.to_string()));
}

fn add_to_commands(data: &str) {
    lu!(COMMANDS).push(data.to_string());
}

fn add_to_readme_manpath(data: &str) {
    lu!(README_MANPATH).push(data.to_string());
}

fn add_to_readme_path(data: &str) {
    lu!(README_PATH).push(data.to_string());
}

fn add_to_info_general(data: &str) {
    lu!(INFO_GENERAL).push(data.to_string());
}

fn add_to_sources(data: &str) {
    lu!(SOURCES).push(data.to_string());
}

fn add_to_info_path(data: &str) {
    lu!(INFO_PATH).push(data.to_string());
}

fn add_to_info_ld_library_path(data: &str) {
    lu!(INFO_LD_LIBRARY_PATH).push(data.to_string());
}

fn add_to_info_pythonpath(data: &str) {
    lu!(INFO_PYTHONPATH).push(data.to_string());
}

fn add_to_info_perl5lib(data: &str) {
    lu!(INFO_PERL5LIB).push(data.to_string());
}

fn add_to_load(data: String) {
    lu!(LOAD).push(data);
}

// functions for load and unload
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn getenv(var: String) -> String {
    match env::var(&var) {
        Ok(res) => res,
        Err(_) => {
            show_warning!("${} not found", var);
            String::from("")
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn print(msg: String) {
    eprintln!("{}", msg);
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn source(wanted_shell: String, path: String) {
    let (shell, _) = get_shell_info();
    if shell == wanted_shell {
        add_to_commands(&format!("source \"{}\"", path));
    }
}

fn info_bin(bin: String) {
    lu!(INFO_BIN).push(bin.to_string());
}

// stub functions for unloading
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn unsetenv_stub(var: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn remove_path_stub(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn system_stub(cmd: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn load_stub(module: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn conflict_stub(module: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn deprecated_stub(time: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn unload_stub(module: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn description_stub(desc: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn getenv_stub(var: String) -> String {
    String::new()
}
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn print_stub(_msg: String) {}
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn source_stub(_wanted_shell: String, _path: String) {}
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn info_bin_stub(_bin: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn prepend_path_stub(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn append_path_stub(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn setenv_stub(var: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn set_alias_stub(name: String, val: String) {}
#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
pub fn is_loaded_stub(var: String) -> bool {
    true
}

// unload functions

#[allow(unused_variables)]
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn setenv_unload(var: String, val: String) {
    unsetenv(var);
}

// readme functions
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn readme_path(var: String, val: String) {
    if var == "PATH" {
        add_to_readme_path(&val);
    } else if var == "MANPATH" {
        add_to_readme_manpath(&val);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn setenv_readme(var: String, val: String) {
    if var == "MANPATH" {
        add_to_readme_manpath(&val);
    } else if var == "PATH" {
        add_to_readme_path(&val);
    }
}

// info functions

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn setenv_info(var: String, val: String) {
    add_to_info_general(&format!("{}={}", var, val));
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
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
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
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

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn deprecated_info(time: String) {
    let now = Utc::now().timestamp_millis();

    let mstime = format!("{} 00:00:00 +0000", time);
    let mstime = match DateTime::parse_from_str(&mstime, "%Y-%m-%d %T %z") {
        Ok(mstime) => mstime,
        Err(e) => {
            show_warning!("Error parsing deprecated time argument: {}", e);
            return;
        }
    };
    let mstime = mstime.timestamp_millis();

    let mut deprecated = lu!(DEPRECATED);
    if now > mstime {
        lu!(INFO_DEPRECATED).push(format!(
            "\n   This module was removed at {} and cannot be used anymore.",
            time
        ));
        *deprecated = Deprecated::from(String::new(), time, DeprecatedState::After);
    } else {
        lu!(INFO_DEPRECATED).push(format!(
            "This has been marked as deprecated and will be removed after {}.\n",
            time
        ));
        *deprecated = Deprecated::from(String::new(), time, DeprecatedState::Before);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn source_info(wanted_shell: String, path: String) {
    let (shell, _) = get_shell_info();
    if shell == wanted_shell {
        add_to_sources(&path);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn load_info(module: String) {
    add_to_load(module);
}
// load functions

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn setenv(var: String, val: String) {
    add_to_env_vars(&var, &val);
    env::set_var(&var, val);
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn is_loaded(var: String) -> bool {
    super::is_module_loaded(&var, false)
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn unsetenv(var: String) {
    let (shell, _) = get_shell_info();
    if shell == "bash" || shell == "zsh" {
        add_to_commands(&format!("unset \"{}\"", var));
    } else if shell == "perl" {
        add_to_commands(&format!("undef \"{}\"", var));
    } else if shell == "python" {
        add_to_commands(&format!("os.environ[\"{}\"] = \"\";", var));
        add_to_commands(&format!("del os.environ[\"{}\"];", var));
    } else if shell == "r" {
        add_to_commands(&format!("Sys.unsetenv(\"{}\")", var));
    } else {
        add_to_commands(&format!("unsetenv \"{}\"", var));
    }
    env::remove_var(&var);
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
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

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
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

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
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

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn unset_alias(name: String, val: String) {
    let (shell, _) = get_shell_info();
    if shell == "bash" || shell == "zsh" {
        add_to_commands(&format!("unalias \"{}\"", name));
    } else if shell == "tcsh" || shell == "csh" {
        add_to_commands(&format!("unalias \"{}={}\"", name, val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn set_alias(name: String, val: String) {
    let (shell, _) = get_shell_info();
    if shell != "python" && shell != "perl" {
        add_to_commands(&format!("alias {}=\"{}\"", name, val));
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn system(cmd: String) {
    let (shell, _) = get_shell_info();
    if shell != "python" && shell != "perl" {
        add_to_commands(&cmd);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn system_unload(cmd: String) {
    let (shell, _) = get_shell_info();
    if shell != "python" && shell != "perl" {
        add_to_commands(&cmd);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
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

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn deprecated(time: String) {
    let now = Utc::now().timestamp_millis();

    let mstime = format!("{} 00:00:00 +0000", time);
    let mstime = match DateTime::parse_from_str(&mstime, "%Y-%m-%d %T %z") {
        Ok(mstime) => mstime,
        Err(e) => {
            show_warning!("Error parsing deprecated time argument: {}", e);
            return;
        }
    };
    let mstime = mstime.timestamp_millis();

    let mut deprecated = lu!(DEPRECATED);
    if now > mstime {
        *deprecated = Deprecated::from(String::new(), time, DeprecatedState::After);
    } else {
        *deprecated = Deprecated::from(String::new(), time, DeprecatedState::Before);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn conflict(module: String) {
    if super::is_module_loaded(module.as_ref(), false) {
        let (shell, _) = get_shell_info();

        let spaces = if shell == "noshell" || shell == "perl" || shell == "python" || shell == "r" {
            ""
        } else {
            "  "
        };

        let bold_module = bold(&shell, &module);

        let shell = &shell;
        if shell != "noshell" {
            echo("", shell);
        }
        echo(
            &format!("{}Cannot continue because the module {} is loaded.", spaces, bold_module),
            shell,
        );

        if shell != "noshell" {
            echo(
                &format!("{}You'll need to unload {} before you can continue:", spaces, bold_module),
                shell,
            );
            echo("", shell);
            echo(&format!("{}module unload {}", spaces, bold_module), shell);
            echo("", shell);
        }
        CONFLICT.store(true, Ordering::Relaxed);
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn unload(module: String) {
    let (shell, _) = get_shell_info();
    let shell: &String = &shell;
    let modulepaths = super::get_module_paths(false);
    let mut rsmod_command: Rsmodule = Rsmodule {
        cmd: "unload",
        typed_command: "unload",
        arg: &module,
        search_path: &modulepaths,
        shell,
        shell_width: 80,
    };
    super::command(&mut rsmod_command);
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn description(desc: String) {
    lu!(INFO_DESCRIPTION).push(desc.replace("\"", "\\\""));
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
fn description_cache(desc: String) {
    add_to_info_general(&desc);
}

pub fn register_stub_fn(engine: &mut Engine) {
    engine.register_fn("setenv", setenv_stub);
    engine.register_fn("unsetenv", unsetenv_stub);
    engine.register_fn("prepend_path", prepend_path_stub);
    engine.register_fn("append_path", append_path_stub);
    engine.register_fn("remove_path", remove_path_stub);
    engine.register_fn("system", system_stub);
    engine.register_fn("system_unload", system_stub);
    engine.register_fn("load", load_stub);
    engine.register_fn("conflict", conflict_stub);
    engine.register_fn("deprecated", deprecated_stub);
    engine.register_fn("unload", unload_stub);
    engine.register_fn("getenv", getenv_stub);
    engine.register_fn("description", description_stub);
    engine.register_fn("set_alias", set_alias_stub);
    engine.register_fn("is_loaded", is_loaded_stub);
    engine.register_fn("print", print_stub);
    engine.register_fn("source", source_stub);
    engine.register_fn("add_bin_to_info", info_bin_stub);
}

pub fn run(path: &PathBuf, action: &str) {
    let mut engine = Engine::new();

    register_stub_fn(&mut engine);
    init_vars_and_commands();

    if action == "unload" {
        // for unloading, we swap some functions
        // prepand_path and append_path are just remove_path
        // setenv should be an alternative to unsetenv
        // the others arent used
        engine.register_fn("setenv", setenv_unload);
        engine.register_fn("prepend_path", remove_path);
        engine.register_fn("append_path", remove_path);
        engine.register_fn("system_unload", system_unload);
        engine.register_fn("set_alias", unset_alias);
    } else if action == "load" {
        engine.register_fn("setenv", setenv);
        engine.register_fn("unsetenv", unsetenv);
        engine.register_fn("prepend_path", prepend_path);
        engine.register_fn("append_path", append_path);
        engine.register_fn("remove_path", remove_path);
        engine.register_fn("system", system);
        engine.register_fn("load", load);
        engine.register_fn("conflict", conflict);
        engine.register_fn("deprecated", deprecated);
        engine.register_fn("unload", unload);
        engine.register_fn("getenv", getenv);
        engine.register_fn("set_alias", set_alias);
        engine.register_fn("is_loaded", is_loaded);
        engine.register_fn("print", print);
        engine.register_fn("source", source);
    } else if action == "info" {
        engine.register_fn("setenv", setenv_info);
        engine.register_fn("prepend_path", prepend_path_info);
        engine.register_fn("append_path", append_path_info);
        engine.register_fn("load", load_info);
        engine.register_fn("deprecated", deprecated_info);
        engine.register_fn("description", description);
        engine.register_fn("is_loaded", is_loaded);
        engine.register_fn("source", source_info);
        engine.register_fn("add_bin_to_info", info_bin);
    } else if action == "description" {
        engine.register_fn("description", description_cache);
        engine.register_fn("set_alias", set_alias);
        engine.register_fn("is_loaded", is_loaded);
    } else if action == "readme" || action == "cd" {
        engine.register_fn("setenv", setenv_readme);
        engine.register_fn("prepend_path", readme_path);
        engine.register_fn("append_path", readme_path);
        engine.register_fn("set_alias", set_alias);
        engine.register_fn("is_loaded", is_loaded);
    } else if action == "deprecated" {
        engine.register_fn("deprecated", deprecated);
    }

    // FIXME: this error is vague when a module exists in the cache but not on disk
    match engine.eval_file::<String>(path.to_string_lossy().into_owned().as_ref()) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            // if e.to_string() != "Cast of output failed" {
            if !e.to_string().starts_with("Cast of output failed") {
                show_warning!(
                    "modulescript error: {} ({})",
                    e.to_string(),
                    path.to_string_lossy().into_owned()
                );
            }
        }
    }
}

pub fn get_readme_paths() -> Vec<String> {
    let paths: Vec<String> = lu!(README_PATH).to_vec();

    paths
}

pub fn get_readme_manpaths() -> Vec<String> {
    let paths: Vec<String> = lu!(README_MANPATH).to_vec();

    paths
}

pub fn get_description() -> Vec<String> {
    let mut output: Vec<String> = Vec::new();

    // there can be multiple description calls, but
    // only store the first line of the description in
    // the cache file
    let desc: Vec<String> = lu!(INFO_GENERAL).to_vec();
    if !desc.is_empty() {
        output.push(lu!(INFO_GENERAL).get(0).unwrap().to_string());
    }

    output
}

pub fn get_output(selected_module: &str, action: &str, shell: &str) -> Vec<String> {
    if CONFLICT.load(Ordering::Relaxed) {
        return Vec::new();
    }

    // don't load, this module is deprecated
    let deprecated = lu!(DEPRECATED);
    match deprecated.state {
        DeprecatedState::Not => {}
        DeprecatedState::Before => eprintln!(
            "\n  The module '{}' has been marked as deprecated and will be removed after {}.\n",
            bold(shell, selected_module),
            bold(shell, &deprecated.time)
        ),
        DeprecatedState::After => {
            eprintln!(
                "\n  The module '{}' was removed at {} and cannot be used anymore.\n",
                bold(shell, selected_module),
                bold(shell, &deprecated.time)
            );
            return Vec::new();
        }
    }

    if action == "unload" {
        remove_path(super::ENV_LOADEDMODULES.to_string(), selected_module.to_string());
    } else if action == "load" {
        prepend_path(super::ENV_LOADEDMODULES.to_string(), selected_module.to_string());
    }

    // this part must be below the above part
    let mut output: Vec<String> = Vec::new();

    for result in lu!(ENV_VARS).iter() {
        let mut data: String = String::new();
        if shell == "bash" || shell == "zsh" {
            data = format!("export {}=\"{}\"", result.0, result.1);
        } else if shell == "tcsh" || shell == "csh" {
            data = format!("setenv {} \"{}\"", result.0, result.1);
        } else if shell == "python" {
            data = format!("os.environ[\"{}\"] = \"{}\";", result.0, result.1);
        } else if shell == "r" {
            // todo
            // if result.0 = LD_LIBRARY_PATH
            // loop through that folder and for each *.so
            // dyn.load('lib.so')
            data = format!("Sys.setenv({} = \"{}\")", result.0, result.1);
        } else if shell == "perl" {
            data = format!("$ENV{{{}}}=\"{}\";", result.0, result.1);
        }

        if shell != "noshell" {
            output.push(data);
        }
    }

    for line in lu!(COMMANDS).iter() {
        if shell == "r" {
            output.push(format!("system(\"{}\")", line.to_string()));
        } else {
            output.push(line.to_string());
        }
    }

    output
}

// this function prints information about the module
pub fn get_info(shell: &str, module: &str) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut got_output: bool = false;
    let is_deprecated = is_deprecated();

    let tmp = format!("= {} =", module);
    let title_bold_module = bold(shell, &tmp);

    //output.push(format!("echo \"{:=^1$}\"", module.to_string(), module.len()+5));
    if !is_deprecated {
        output.push(format!("echo \"{}\"", bold(shell, &"=".repeat(module.len() + 4))));

        output.push(format!("echo \"{}\"", title_bold_module.to_string()));
        output.push(format!("echo \"{}\"", bold(shell, &"=".repeat(module.len() + 4))));
        output.push(String::from("echo \"\""));
    }

    if lu!(INFO_DEPRECATED).iter().len() > 0 {
        got_output = true;
    }
    for line in lu!(INFO_DEPRECATED).iter() {
        let line = bold(shell, line);
        if shell == "bash" || shell == "zsh" {
            output.push(format!("echo $\"{}\"", line.to_string()));
        } else if shell == "csh" || shell == "tcsh" {
            output.push(format!("echo \"{}\"", line.to_string().replace("\n", "\\n")));
        } else {
            output.push(format!("echo \"{}\"", line.to_string()));
        }
    }

    if !is_deprecated {
        if lu!(INFO_DESCRIPTION).iter().len() > 0 {
            got_output = true;
        }
        for line in lu!(INFO_DESCRIPTION).iter() {
            if shell == "bash" || shell == "zsh" {
                output.push(format!("echo $\"{}\"", line.to_string()));
            } else if shell == "csh" || shell == "tcsh" {
                output.push(format!("echo \"{}\"", line.to_string().replace("\n", "\\n")));
            } else {
                output.push(format!("echo \"{}\"", line.to_string()));
            }
        }

        if lu!(INFO_GENERAL).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "Sets the following variables: ")));
            got_output = true;
        }
        for line in lu!(INFO_GENERAL).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }

        if lu!(SOURCES).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "Sources the following files:")));
            got_output = true;
        }
        for line in lu!(SOURCES).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }
        // TODO: find man pages and let the user know

        if lu!(INFO_PATH).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "Executables can be found in: ")));
            got_output = true;
        }
        for line in lu!(INFO_PATH).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }

        if lu!(INFO_LD_LIBRARY_PATH).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "Libraries can be found in: ")));
            got_output = true;
        }
        for line in lu!(INFO_LD_LIBRARY_PATH).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }

        if lu!(INFO_PYTHONPATH).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "\\$PYTHONPATH: ")));
            got_output = true;
        }
        for line in lu!(INFO_PYTHONPATH).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }

        if lu!(INFO_PERL5LIB).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "\\$PERL5LIB: ")));
            got_output = true;
        }
        for line in lu!(INFO_PERL5LIB).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }

        if lu!(LOAD).iter().len() > 0 {
            output.push("echo \"\"".to_string());
            output.push(format!("echo \"{}\"", bold(shell, "Depends on: ")));
            got_output = true;
        }
        for line in lu!(LOAD).iter() {
            output.push(format!("echo '{}'", line.to_string()));
        }

        let mut execs: Vec<String> = Vec::new();
        let mut filtered: bool = false;
        if lu!(INFO_BIN).is_empty() || env::var("RSMODULES_DONT_FILTER_INFO").is_ok() {
            for line in lu!(INFO_PATH).iter() {
                if Path::new(line).is_dir() {
                    // if activate, activate.csh, activate.fish and activate_this.py exist
                    // then we are in a python virtualenv, we can skip the typical python
                    // binaries, we don't want to see them when we run 'module info program/arch/version'

                    let is_virtual_env = is_virtual_env(PathBuf::from(line));

                    let entries = match read_dir(line) {
                        Ok(entry) => entry,
                        Err(_) => continue,
                    };

                    for entry in entries {
                        let path = match &entry {
                            Ok(p) => p.path(),
                            Err(_) => continue,
                        };

                        let file_name = match &entry {
                            Ok(p) => p.file_name(),
                            Err(_) => continue,
                        };

                        if is_python_binary(file_name) && is_virtual_env {
                            continue;
                        }

                        if path.is_dir() {
                            continue;
                        }

                        if path.is_executable() {
                            execs.push(format!("echo '{}'", strip_dir(path.to_str().unwrap())));
                            got_output = true;
                        }
                    }
                }
            }
        } else {
            let bins: Vec<String> = lu!(INFO_BIN).to_vec();
            for bin in bins {
                execs.push(format!("echo '{}'", bin));
                got_output = true;
                filtered = true;
            }
        }

        if !execs.is_empty() {
            output.push(String::from("echo ''"));
            if execs.len() > 1 {
                output.push(format!(
                    "echo \"{}\"",
                    bold(shell, "Try one of these commands to run the program: ")
                ));
            } else {
                output.push(format!("echo \"{}\"", bold(shell, "Try this command to run the program: ")));
            }
        }

        execs.sort();
        for exec in execs {
            output.push(exec);
        }

        if filtered {
            output.push(String::from("echo ''"));
            output.push(String::from("echo 'Some binaries are omitted in this output'"));
            output.push(String::from(
                "echo 'Set the environment var RSMODULES_DONT_FILTER_INFO if you want unfiltered output'",
            ));
        }
    }

    if got_output {
        output.push(String::from("echo ''"));
    }

    output
}

// returns true if the deprecated AFTER state has been reached
fn is_deprecated() -> bool {
    let is_deprecated: bool;
    {
        // we ne need a different scope, or DEPRECATED is locked
        let deprecated = lu!(DEPRECATED);
        match deprecated.state {
            DeprecatedState::After => {
                is_deprecated = true;
            }
            DeprecatedState::Before => {
                is_deprecated = false;
            }
            DeprecatedState::Not => {
                is_deprecated = false;
            }
        }
    }
    is_deprecated
}

fn is_virtual_env(path: PathBuf) -> bool {
    if env::var("RSMODULES_DONT_FILTER_INFO").is_ok() {
        return false;
    }

    let mut tmp_path = path;
    tmp_path.push("bin");

    let files = vec!["activate", "activate.csh", "activate.fish", "activate_this.py"];
    let mut counter = 0;

    for file in &files {
        tmp_path.set_file_name(file);
        if tmp_path.exists() {
            counter += 1;
        }
    }

    if counter == files.len() {
        return true;
    }

    false
}

fn is_python_binary(file_name: OsString) -> bool {
    let files = vec![
        "^activate$",
        "^activate.csh$",
        "^activate.fish$",
        "^activate_this.py$",
        "^easy_install$",
        "^easy_install[0-9]$",
        "^easy_install[0-9].[0-9]$",
        "^easy_install-[0-9].[0-9]$",
        "^pip$",
        "^pip[0-9]$",
        "^pip[0-9].[0-9]$",
        "^pip-[0-9].[0-9]$",
        "^f2py$",
        "^f2py[0-9]$",
        "^f2py[0-9].[0-9]$",
        "^f2py-[0-9].[0-9]$",
        "^python$",
        "^python[0-9]$",
        "^python[0-9].[0-9]$",
        "^python-[0-9].[0-9]$",
        "^python-config$",
        "^wheel$",
        "^wheel[0-9]$",
        "^wheel[0-9].[0-9]$",
        "^wheel-[0-9].[0-9]$",
    ];

    for file in files {
        let re = Regex::new(file).unwrap();

        if re.is_match(file_name.to_str().unwrap()) {
            return true;
        }
    }

    false
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
