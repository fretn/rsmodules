use std::fs::File;
use std::path::{Path,PathBuf};
use std::io::{BufReader, BufRead, Write};
use std::env;

pub struct Rmodule<'a> {
    pub command: &'a str,
    pub module: &'a str,
    pub modules: &'a Vec<String>,
    pub modulepaths: &'a Vec<String>,
    pub shell: &'a str,
    pub tmpfile: &'a mut File,
}

// bad function name, maybe I should split this on two functions
pub fn get_module_list() -> (Vec<String>, Vec<String>) {
    let mut modulepath: String = String::from("/usr/local");
    let mut modules: Vec<String> = Vec::new();
    let mut found_cachefile: bool = false;
    let mut modulepaths: Vec<String> = Vec::new();

    match env::var("MODULEPATH") {
        Ok(path) => modulepath = path,
        Err(_) => {
            show_warning!("$MODULEPATH not found, using {}", modulepath);
        }
    };

    //println!("modulepath: {}", modulepath);
    let modulepath: Vec<&str> = modulepath.split(':').collect();
    for path in modulepath {
        // test if cachefiles exist in the paths
        // if they don't and we have write permission in that folder
        // we should create the cache
        modulepaths.push(path.to_string());
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
    return (modules, modulepaths);
}

pub fn command(mut rmodule: Rmodule) {

    if rmodule.command == "load" {
        load(&mut rmodule);
    } else if rmodule.command == "unload" {
        unload(rmodule.module, rmodule.shell);
    } else if rmodule.command == "available" {
        available(rmodule.module, rmodule.modules, &rmodule.tmpfile);
    } else if rmodule.command == "list" {
        list(rmodule.module, rmodule.shell);
    } else if rmodule.command == "purge" {
        purge(rmodule.module, rmodule.shell);
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

fn execute_command_and_parse() {
    // we basicly execute a system call (sourcing two bash files, one of them is the module)
    // then we parse the output of the call (the output of env)
    // and replace all the variables BLAH=124
    // with export|setenv BLAH(=)124
}

fn parse_modulefile() -> bool {
    return true;
    // execute_command_and_parse(strva(". /home/frlae/svn/frlae/modules/module_load_tools.sh &&
    //. %s && env", path), &buffer, shell);
}

//fn load(module: &str, modulepaths: &Vec<String>, shell: &str) {
fn load(rmodule: &mut Rmodule) {
    //println_stderr!("load {} {}", module, shell);
    //println_stderr!("{:?}", modulepaths);

    let (mut reversed_modules, _) = get_module_list();
    reversed_modules.reverse();

    'outer: for modulepath in rmodule.modulepaths {
        let testpath = format!("{}/{}", modulepath, rmodule.module);
        if Path::new(&testpath).exists() {
            // we got it, now we need to figure out if its a partial match or not
            if Path::new(&testpath).is_file() {
                println_stderr!("full match: {}", testpath); 
            } else {
                println_stderr!("partial match: {}", testpath); 
                for module in &reversed_modules {
                    if module.starts_with(rmodule.module) {
                        println_stderr!("{}", module);
                        break 'outer;
                    }
                }
                // we got a partial match, now we need to find the default module
                // for this folder or subfolders
                // loop through all the modules and get the first one
                // that matches starts_with
            }
        } 
    }

    // check if module file exists
    // run over modulepaths, check if a folder/file exists with the wanted 'module' var

    // if not, maybe check if its a partial match
    // blast -> blast/x86_64/1.0 and blast/x86_64/2.0
    // then we need to load the Default version
    // or just the latest one

    // check if we are already loaded (LOADEDMODULES env var)

    // we already know the path to the module file (see above)
    // parse the module file and if successful
    // add it to the LOADEDMODULES env var
    // else unload the module

}

fn unload(module: &str, shell: &str) {
    println_stderr!("echo 'unload {} {}'", module, shell);
}

fn available(module: &str, modules: &Vec<String>, mut tmpfile: &File) {

    for avmodule in modules {
        if module != "" {
            let avmodule_lc: String = avmodule.to_lowercase();
            let module_lc: String = module.to_lowercase();
            let avmodule_lc: &str = avmodule_lc.as_ref();
            let module_lc: &str = module_lc.as_ref();

            // contains is case sensitive, lowercase
            // everything
            // TODO: colored output
            if avmodule_lc.contains(module_lc) {
                write_av_output(&avmodule, &mut tmpfile);
            }
        } else {
            write_av_output(&avmodule, &mut tmpfile);
        }
    }
}

fn write_av_output(line: &str, mut tmpfile: &File) {
    let data = format!("echo '{}'\n", line);
    tmpfile.write_all(data.as_bytes()).expect("Unable to write data");
    tmpfile.write_all("\n".as_bytes()).expect("Unable to write data");
}

fn list(module: &str, shell: &str) {
    println_stderr!("echo 'list {} {}'", module, shell);
}

fn purge(module: &str, shell: &str) {
    println_stderr!("echo 'purge {} {}'", module, shell);
}
