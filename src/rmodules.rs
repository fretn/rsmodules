use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufRead, Write};
use std::env;
use std::process::Command;

static DEFAULT_MODULE_PATH: &'static str = "/usr/local";
static ENV_LOADEDMODULES: &'static str = "LOADEDMODULES"; // name of an env var

pub struct Rmodule<'a> {
    pub cmd: &'a str, // load|list|avail|...
    pub arg: &'a str, // blast/12.1 | blast | blast/12
    pub list: &'a Vec<String>, // list of all av modules
    pub search_path: &'a Vec<String>, // module paths
    pub shell: &'a str, // tcsh|csh|bash|zsh
    pub tmpfile: &'a File, // tempfile that will be sourced
    pub installdir: &'a str, // installation folder
}

pub fn get_module_paths() -> Vec<String> {
    let mut modulepath: String = String::from(DEFAULT_MODULE_PATH);
    let mut modulepaths: Vec<String> = Vec::new();

    match env::var("MODULEPATH") {
        Ok(path) => modulepath = path,
        Err(_) => {
            show_warning!("$MODULEPATH not found, using {}", modulepath);
        }
    };

    let modulepath: Vec<&str> = modulepath.split(':').collect();
    for path in modulepath {
        modulepaths.push(path.to_string());
    }

    return modulepaths;
}

pub fn get_module_list() -> Vec<String> {
    let mut modules: Vec<String> = Vec::new();
    let mut found_cachefile: bool = false;
    let modulepaths = get_module_paths();
    for path in modulepaths {
        // test if cachefiles exist in the paths
        // if they don't and we have write permission in that folder
        // we should create the cache
        let mut testpath = PathBuf::from(path);
        testpath.push(".modulesindex");

        if testpath.exists() {
            parse_modules_cache_file(&testpath, &mut modules);
            found_cachefile = true;
        } else {
            show_warning!("Cache file: {} doesn't exist.", testpath.display());
            // TODO: generate cache
        }
    }

    if !found_cachefile {
        crash!(1, "No cachefiles found.");
    }

    modules.sort();
    return modules;
}

pub fn command(rmod: &mut Rmodule) {

    if rmod.cmd == "load" {
        load(rmod);
    } else if rmod.cmd == "unload" {
        unload(rmod);
    } else if rmod.cmd == "available" {
        available(rmod);
    } else if rmod.cmd == "list" {
        list(rmod);
    } else if rmod.cmd == "purge" {
        purge(rmod);
    }
}

fn parse_modules_cache_file(filename: &PathBuf, modules: &mut Vec<String>) {

    // read filename line by line, and push it to modules
    let file = BufReader::new(File::open(filename).unwrap());
    for (_, line) in file.lines().enumerate() {
        let buffer = line.unwrap();
        modules.push(String::from(buffer));
    }
}

fn run_modulefile(path: &PathBuf, rmod: &mut Rmodule) -> bool {
    let cmd = format!(". {0}/module_load_tools.sh && . {1} && env",
                      rmod.installdir,
                      path.to_str().unwrap());

    let output = Command::new("bash")
        .args(&["-c", cmd.as_ref()])
        .output()
        .expect("failed to execute process");

    let mut output = String::from_utf8_lossy(&output.stdout);
    let output = output.to_mut();

    let output: Vec<&str> = output.split('\n').collect();
    for line in output {
        if line != "" {
            let split: Vec<&str> = line.splitn(2, '=').collect();
            if split.len() < 2 {
                print_unset_env_var(split[0], rmod)
            } else if split.len() == 2 {
                print_set_env_var(split[0], split[1], rmod)
            } else {
                crash!(1,
                       "Failed to load modulefile, something in your env breaks rmodules");
            }
        }
    }

    return true;
}

fn print_unset_env_var(name: &str, rmod: &mut Rmodule) {
    let data: String;

    if rmod.shell == "bash" || rmod.shell == "zsh" {
        data = format!("unset {0}\n", name);
    } else {
        data = format!("unsetenv {0}\n", name);
    }

    crash_if_err!(1, rmod.tmpfile.write_all(data.as_bytes()));
}

fn print_set_env_var(name: &str, value: &str, rmod: &mut Rmodule) {
    let data: String;

    if rmod.shell == "bash" || rmod.shell == "zsh" {
        data = format!("export {0}='{1}'\n", name, value);
    } else {
        data = format!("setenv {0} '{1}'\n", name, value);
    }

    crash_if_err!(1, rmod.tmpfile.write_all(data.as_bytes()));
}

fn load(rmod: &mut Rmodule) {

    let mut reversed_modules = get_module_list();
    reversed_modules.reverse();
    let mut selected_module = rmod.arg;
    let mut modulefile: PathBuf = PathBuf::new();
    let mut found: bool = false;

    // check if module file exists
    // run over modulepaths, check if a folder/file exists with the wanted 'module' var

    // if not, maybe check if its a partial match
    // blast -> blast/x86_64/1.0 and blast/x86_64/2.0
    // then we need to load the Default version
    // or just the latest one

    'outer: for modulepath in rmod.search_path {
        let testpath = format!("{}/{}", modulepath, rmod.arg);
        if Path::new(&testpath).exists() {

            // we got it, now we need to figure out if its a partial match or not
            if Path::new(&testpath).is_file() {
                found = true;
                modulefile = PathBuf::from(&testpath);
            } else {
                for module in &reversed_modules {

                    // we got a partial match, now we need to find the default module
                    // for this folder or subfolders
                    // loop through all the modules and get the first one
                    // that matches starts_with

                    // partial matches only work for file/folder names
                    // blast or blast/x86_64 but not blas or blast/x86_
                    // because of the above 'exists()' check

                    if module.starts_with(rmod.arg) {
                        //println_stderr!("{}", module);
                        selected_module = module;
                        found = true;
                        let testpath = format!("{}/{}", modulepath, module);
                        modulefile = PathBuf::from(&testpath);

                        // break out of the outer loop
                        break 'outer;
                    }
                }
            }
        }
    }

    if !found {
        crash!(1, "Module {0} not found.", selected_module);
    }

    // check if we are already loaded (LOADEDMODULES env var)
    if is_module_loaded(selected_module) {
        // unload the module and then load it again ??
        return;
    }

    // we already know the path to the module file (see above)
    // parse the module file and if successful
    // add it to the LOADEDMODULES env var
    // else unload the module
    if run_modulefile(&modulefile, rmod) {
        add_module_to_loadedmodules(selected_module, rmod);
    }

}

fn add_module_to_loadedmodules(name: &str, rmod: &mut Rmodule) {
    let loadedmodules: String;
    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            print_set_env_var(ENV_LOADEDMODULES, name, rmod);
            return;
        }
    };


    let mut loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    loadedmodules.push(name);
    let loaded_modules = loadedmodules.join(":");

    print_set_env_var(ENV_LOADEDMODULES, loaded_modules.as_ref(), rmod);

}

fn is_module_loaded(name: &str) -> bool {
    let loadedmodules: String;
    match env::var(ENV_LOADEDMODULES) {
        Ok(list) => loadedmodules = list,
        Err(_) => {
            return false;
        }
    };

    let loadedmodules: Vec<&str> = loadedmodules.split(':').collect();
    for module in loadedmodules {
        if module == name {
            return true;
        }
    }

    return false;
}

fn unload(rmod: &mut Rmodule) {
    println_stderr!("echo 'unload {} {}'", rmod.arg, rmod.shell);
}

//fn available(module: &str, modules: &Vec<String>, mut tmpfile: &File) {
fn available(rmod: &mut Rmodule) {

    for avmodule in rmod.list {
        if rmod.arg != "" {
            let avmodule_lc: String = avmodule.to_lowercase();
            let module_lc: String = rmod.arg.to_lowercase();
            let avmodule_lc: &str = avmodule_lc.as_ref();
            let module_lc: &str = module_lc.as_ref();

            // contains is case sensitive, lowercase
            // everything
            // TODO: colored output
            if avmodule_lc.contains(module_lc) {
                write_av_output(&avmodule, &mut rmod.tmpfile);
            }
        } else {
            write_av_output(&avmodule, &mut rmod.tmpfile);
        }
    }
}

fn write_av_output(line: &str, mut tmpfile: &File) {
    let data = format!("echo '{}'\n", line);
    tmpfile.write_all(data.as_bytes()).expect("Unable to write data");
    tmpfile.write_all("\n".as_bytes()).expect("Unable to write data");
}

fn list(rmod: &mut Rmodule) {
    println_stderr!("echo 'list {} {}'", rmod.arg, rmod.shell);
}

fn purge(rmod: &mut Rmodule) {
    println_stderr!("echo 'purge {} {}'", rmod.arg, rmod.shell);
}
