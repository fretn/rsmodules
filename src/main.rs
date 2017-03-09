extern crate tempfile;

#[path = "rmodules.rs"]
mod rmod;

use std::io::{BufReader, BufRead, Write};
use std::env;
use std::fs::File;
use std::path::PathBuf;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);


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

fn print_usage(shell_error: bool, inside_eval: bool) {

    let error_msg: &str = "Usage: rmodules <shell> <load|unload|list|purge|available> [module \
                           name]";
    // TODO: get shells from the shell supported vec
    let shell_error_msg: &str = "Only tcsh and bash are supported";

    if inside_eval {
        println!("echo '{}'", &error_msg);
    } else {
        println_stderr!("{}", &error_msg);
    }

    if shell_error == true {
        if inside_eval {
            println!("echo '{}'", &shell_error_msg);
        } else {
            println_stderr!("{}", &shell_error_msg);
        }
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

fn init_modules_path() -> Vec<String> {
    let mut modulepath: String = String::from("/usr/local");
    let mut modules: Vec<String> = Vec::new();

    match env::var("MODULEPATH") {
        Ok(path) => modulepath = path,
        Err(e) => {
            println_stderr!("Error: {}\n$MODULEPATH not found, falling back to {}",
                            e,
                            modulepath)
        }
    };

    //println!("modulepath: {}", modulepath);
    let modulepath: Vec<&str> = modulepath.split(':').collect();
    for path in modulepath {
        // test if cachefiles exist in the paths
        // if they don't and we have write permission in that folder
        // we should create the cache
        let mut testpath = PathBuf::from(path);
        testpath.push(".modulesindex");
        if testpath.exists() {
            parse_modules_cache_file(&testpath, &mut modules);
        } else {
            println_stderr!("Cache file: {} doesn't exist.", testpath.display());
            // TODO: generate cache
        }
    }

    return modules;
}

fn parse_commandline(args: &Vec<String>, modules: &Vec<String>) -> bool {
    let shell: &str = &args[1];
    let command: &str;
    let modulename: &str;

    if !is_shell_supported(shell) {
        print_usage(true, true);
        return false;
    }

    if args.len() >= 3 {
        command = &args[2];

        if command == "load" || command == "unload" || command == "available" {
            if args.len() > 3 {
                modulename = &args[3];
                rmod::load(modulename, shell);
                //run_command(command, modulename);
            } else {
                print_usage(false, true);
                return false;
            }
        } else if command == "list" || command == "purge" {
            //run_command(command);
        } else {
            print_usage(false, true);
            return false;
        }
    }

    return true;
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 && (&args[1] == "-h" || &args[1] == "--help") {
        print_usage(false, false);
        return;
    } else if args.len() >= 3 && (&args[1] == "-h" || &args[1] == "--help") {
        print_usage(false, true);
        return;
    }

    // create temporary file
    //let mut tmpfile: File = tempfile::tempfile().unwrap();
    let mut tmpfile: File = tempfile::tempfile().expect("failed to create temporary file");

    // parse modules path
    let modules: Vec<String>;
    modules = init_modules_path();

    //println!("{:?}", modules);

    if parse_commandline(&args, &modules) {
        // TODO
        // print 'source tmpfile' or '. tmpfile' to output
        println!("jup done");
    }
}
