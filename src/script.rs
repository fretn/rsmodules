extern crate rhai;
use self::rhai::{Engine, FnRegister};
use std::env;
use std::sync::Mutex;
use std::path::PathBuf;
use std::io::Write;

// WARNING: the scripts don't support tabbed indents in if else structures

lazy_static! {
    static ref ENV_VARS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref COMMANDS: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref CONFLICT: Mutex<bool> = Mutex::new(false);
    static ref SHELL: Mutex<String> = Mutex::new({String::from("bash")});
    static ref INFO_GENERAL: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_LD_LIBRARY_PATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PYTHONPATH: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref INFO_PERL5LIB: Mutex<Vec<String>> = Mutex::new(vec![]);
    static ref DEPEND: Mutex<Vec<String>> = Mutex::new(vec![]);
}

fn init_vars_and_commands() {
    ENV_VARS.lock().unwrap().clear();
    COMMANDS.lock().unwrap().clear();
    INFO_GENERAL.lock().unwrap().clear();
    INFO_PATH.lock().unwrap().clear();
    INFO_LD_LIBRARY_PATH.lock().unwrap().clear();
    INFO_PYTHONPATH.lock().unwrap().clear();
    INFO_PERL5LIB.lock().unwrap().clear();
    DEPEND.lock().unwrap().clear();

    let mut tmp = CONFLICT.lock().unwrap();
    *tmp = false;
}

fn set_shell(shell: &str) {
    let mut tmp = SHELL.lock().unwrap();
    *tmp = shell.to_string();
}

fn add_to_env_vars(data: String) {
    ENV_VARS.lock().unwrap().push(data);
}

fn add_to_commands(data: String) {
    COMMANDS.lock().unwrap().push(data);
}

fn add_to_info_general(data: String) {
    INFO_GENERAL.lock().unwrap().push(data);
}

fn add_to_info_path(data: String) {
    INFO_PATH.lock().unwrap().push(data);
}

fn add_to_info_ld_library_path(data: String) {
    INFO_LD_LIBRARY_PATH.lock().unwrap().push(data);
}

fn add_to_info_pythonpath(data: String) {
    INFO_PYTHONPATH.lock().unwrap().push(data);
}

fn add_to_info_perl5lib(data: String) {
    INFO_PERL5LIB.lock().unwrap().push(data);
}

fn add_to_depend(data: String) {
    DEPEND.lock().unwrap().push(data);
}

// functions for load and unload
fn getenv(var: String) -> String {

    match env::var(&var) {
        Ok(res) => String::from(res),
        Err(_) => {
            show_warning!("${} not found", var);
            String::from("")
        }
    }
}

// dummy functions for unloading
#[allow(unused_variables)]
fn unsetenv_dummy(var: String) {}
#[allow(unused_variables)]
fn remove_path_dummy(var: String, val: String) {}
#[allow(unused_variables)]
fn system_dummy(cmd: String) {}
#[allow(unused_variables)]
fn depend_dummy(module: String) {}
#[allow(unused_variables)]
fn conflict_dummy(module: String) {}
#[allow(unused_variables)]
fn unload_dummy(module: String) {}
#[allow(unused_variables)]
fn description_dummy(desc: String) {}
#[allow(unused_variables)]
fn getenv_dummy(var: String) -> String {
    String::new()
}
#[allow(unused_variables)]
fn prepend_path_dummy(var: String, val: String) {}
#[allow(unused_variables)]
fn append_path_dummy(var: String, val: String) {}
#[allow(unused_variables)]
fn setenv_dummy(var: String, val: String) {}

// unload functions

#[allow(unused_variables)]
fn setenv_unload(var: String, val: String) {
    unsetenv(var);
}

// info functions

fn setenv_info(var: String, val: String) {
    add_to_info_general(format!("{}={}", var, val));
}

fn prepend_path_info(var: String, val: String) {
    if var == "PATH" {
        add_to_info_path(format!("{}", val));
    } else if var == "LD_LIBRARY_PATH" {
        add_to_info_ld_library_path(format!("{}", val));
    } else if var == "PYTHONPATH" {
        add_to_info_pythonpath(format!("{}", val));
    } else if var == "PERL5LIB" {
        add_to_info_perl5lib(format!("{}", val));
    } else {
        add_to_info_general(format!("{}={}", var, val));
    }
}
fn append_path_info(var: String, val: String) {
    if var == "PATH" {
        add_to_info_path(format!("{}", val));
    } else if var == "LD_LIBRARY_PATH" {
        add_to_info_ld_library_path(format!("{}", val));
    } else if var == "PYTHONPATH" {
        add_to_info_pythonpath(format!("{}", val));
    } else if var == "PERL5LIB" {
        add_to_info_perl5lib(format!("{}", val));
    } else {
        add_to_info_general(format!("{}={}", var, val));
    }
}

fn depend_info(module: String) {
    add_to_depend(module);
}
// load functions

fn setenv(var: String, val: String) {
    let shell = SHELL.lock().unwrap();
    if *shell == "bash" || *shell == "zsh" {
        add_to_env_vars(format!("export {}=\"{}\"", var, val));
    } else {
        add_to_env_vars(format!("setenv {} \"{}\"", var, val));
    }
    env::set_var(&var, format!("{}", val));
}

fn unsetenv(var: String) {
    let shell = SHELL.lock().unwrap();
    if *shell == "bash" || *shell == "zsh" {
        add_to_env_vars(format!("unset \"{}\"", var));
    } else {
        add_to_env_vars(format!("unsetenv \"{}\"", var));
    }
    env::remove_var(&var);
}

fn prepend_path(var: String, val: String) {
    let mut current_val: String = String::from("");
    let mut notfound: bool = false;
    let shell = SHELL.lock().unwrap();

    match env::var(&var) {
        Ok(res) => current_val = res,
        Err(_) => {
            //show_warning!("${} not found", var);
            notfound = true;
        }
    };

    if notfound {
        if *shell == "bash" || *shell == "zsh" {
            add_to_env_vars(format!("export {}=\"{}\"", var, val));
        } else {
            add_to_env_vars(format!("setenv {} \"{}\"", var, val));
        }
        env::set_var(&var, format!("{}", val));
    } else {
        if *shell == "bash" || *shell == "zsh" {
            add_to_env_vars(format!("export {}=\"{}:{}\"", var, val, current_val));
        } else {
            add_to_env_vars(format!("setenv {} \"{}:{}\"", var, val, current_val));
        }
        env::set_var(&var, format!("{}:{}", val, current_val));
    }
}

fn append_path(var: String, val: String) {
    let mut current_val: String = String::from("");
    let mut notfound: bool = false;
    let shell = SHELL.lock().unwrap();

    match env::var(&var) {
        Ok(res) => current_val = res,
        Err(_) => {
            //show_warning!("${} not found", var);
            notfound = true;
        }
    };

    if notfound {
        if *shell == "bash" || *shell == "zsh" {
            add_to_env_vars(format!("export {}=\"{}\"", var, val));
        } else {
            add_to_env_vars(format!("setenv {} \"{}\"", var, val));
        }

        env::set_var(&var, format!("{}", val));
    } else {
        if *shell == "bash" || *shell == "zsh" {
            add_to_env_vars(format!("export {}=\"{}:{}\"", var, current_val, val));
        } else {
            add_to_env_vars(format!("setenv {} \"{}:{}\"", var, current_val, val));
        }
        env::set_var(&var, format!("{}:{}", current_val, val));
    }
}

fn remove_path(var: String, val: String) {
    let current_val: String;
    let shell = SHELL.lock().unwrap();

    match env::var(&var) {
        Ok(res) => current_val = res,
        Err(_) => {
            //show_warning!("${} not found", var);
            return;
        }
    };

    let mut values: Vec<&str> = current_val.split(":").collect();
    values.retain(|&x| x != val);

    let result = values.join(":");

    if *shell == "bash" || *shell == "zsh" {
        add_to_env_vars(format!("export {}=\"{}\"", var, result));
    } else {
        add_to_env_vars(format!("export {}=\"{}\"", var, result));
    }
    env::set_var(&var, format!("{}", result));
}

fn system(cmd: String) {
    add_to_commands(cmd);
}

fn depend(module: String) {
    add_to_commands(format!("module load {}", module));
}

fn conflict(module: String) {
    if super::is_module_loaded(module.as_ref()) {
        show_warning!("This module cannot be loaded while {} is loaded.", module);
        let mut data = CONFLICT.lock().unwrap();
        *data = true;
    }
}

fn unload(module: String) {
    add_to_commands(format!("module unload {}", module));
}

fn description(desc: String) {
    println_stderr!("{}", desc);
}

fn description_cache(desc: String) {
    add_to_info_general(desc);
}

pub fn run(path: &PathBuf, selected_module: &str, action: &str, shell: &str) {
    let mut engine = Engine::new();

    init_vars_and_commands();

    set_shell(shell);

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
        engine.register_fn("depend", depend_dummy);
        engine.register_fn("conflict", conflict_dummy);
        engine.register_fn("unload", unload_dummy);
        engine.register_fn("getenv", getenv);
        engine.register_fn("description", description_dummy);

        remove_path(String::from(super::ENV_LOADEDMODULES),
                    String::from(selected_module));
    } else if action == "load" {
        engine.register_fn("setenv", setenv);
        engine.register_fn("unsetenv", unsetenv);
        engine.register_fn("prepend_path", prepend_path);
        engine.register_fn("append_path", append_path);
        engine.register_fn("remove_path", remove_path);
        engine.register_fn("system", system);
        engine.register_fn("depend", depend);
        engine.register_fn("conflict", conflict);
        engine.register_fn("unload", unload);
        engine.register_fn("getenv", getenv);
        engine.register_fn("description", description_dummy);

        prepend_path(String::from(super::ENV_LOADEDMODULES),
                     String::from(selected_module));
    } else if action == "info" {
        engine.register_fn("setenv", setenv_info);
        engine.register_fn("unsetenv", unsetenv_dummy);
        engine.register_fn("prepend_path", prepend_path_info);
        engine.register_fn("append_path", append_path_info);
        engine.register_fn("remove_path", remove_path_dummy);
        engine.register_fn("system", system_dummy);
        engine.register_fn("depend", depend_info);
        engine.register_fn("conflict", conflict_dummy);
        engine.register_fn("unload", unload_dummy);
        engine.register_fn("getenv", getenv_dummy);
        engine.register_fn("description", description);

    } else if action == "description" {
        engine.register_fn("setenv", setenv_dummy);
        engine.register_fn("unsetenv", unsetenv_dummy);
        engine.register_fn("prepend_path", prepend_path_dummy);
        engine.register_fn("append_path", append_path_dummy);
        engine.register_fn("remove_path", remove_path_dummy);
        engine.register_fn("system", system_dummy);
        engine.register_fn("depend", depend_dummy);
        engine.register_fn("conflict", conflict_dummy);
        engine.register_fn("unload", unload_dummy);
        engine.register_fn("getenv", getenv_dummy);
        engine.register_fn("description", description_cache);

    }


    match engine.eval_file::<String>(path.to_string_lossy().into_owned().as_ref()) {
        Ok(result) => println!("{}", result),
        Err(e) => {
            if e.to_string() != "Cast of output failed" {
                show_warning!("modulescript error: {}", e.to_string());
            }
        }
    }
}

pub fn get_description() -> Vec<String> {

    let mut output: Vec<String> = Vec::new();

    for line in INFO_GENERAL.lock().unwrap().iter() {
        output.push(format!("{}", line.to_string()));
    }

    return output;
}

pub fn get_output() -> Vec<String> {
    let data = CONFLICT.lock().unwrap();

    if *data == true {
        return Vec::new();
    }

    let mut output: Vec<String> = Vec::new();

    for line in ENV_VARS.lock().unwrap().iter() {
        output.push(line.to_string());
    }

    for line in COMMANDS.lock().unwrap().iter() {
        output.push(line.to_string());
    }

    return output;
}

// this function prints information about the module
pub fn get_info(shell: &str) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();
    let mut got_output: bool = false;
    let mut bold_start: &str = "$(tput bold)";
    let mut bold_end: &str = "$(tput sgr0)";

    if shell == "tcsh" || shell == "csh" {
        bold_start = "\\033[1m";
        bold_end = "\\033[0m";
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
        output.push(format!("echo \"{}$PYTHONPATH: {}\"", bold_start, bold_end));
        got_output = true;
    }
    for line in INFO_PYTHONPATH.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if INFO_PERL5LIB.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}$PERL5LIB: {}\"", bold_start, bold_end));
        got_output = true;
    }
    for line in INFO_PERL5LIB.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if DEPEND.lock().unwrap().iter().len() > 0 {
        output.push("echo \"\"".to_string());
        output.push(format!("echo \"{}Depends on: {}\"", bold_start, bold_end));
        got_output = true;
    }
    for line in DEPEND.lock().unwrap().iter() {
        output.push(format!("echo '{}'", line.to_string()));
    }

    if got_output {
        output.push(format!("echo ''\n"));
    }

    // TODO: print all executable files in PATH

    return output;
}
