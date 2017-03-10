extern crate rand;
use rand::Rng;

#[path = "rmodules.rs"]
mod rmod;

use std::io::{BufReader, BufRead, Write};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

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

fn print_usage(shell_error: bool) {

    let error_msg: &str = "Usage: rmodules <shell> <load|unload|list|purge|available> [module \
                           name]";
    // TODO: get shells from the shell supported vec
    let shell_error_msg: &str = "Only tcsh and bash are supported";

    println_stderr!("{}", &error_msg);

    if shell_error == true {
        println_stderr!("{}", &shell_error_msg);
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

    modules.sort();
    return modules;
}

fn parse_commandline(args: &Vec<String>, modules: &Vec<String>) {
    let shell: &str = &args[1];
    let command: &str;
    let mut modulename: &str = "";

    // create temporary file in the home folder
    // if the file cannot be created the program panics
    let rstr: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(8)
        .collect();
    let homedir: PathBuf = env::home_dir().expect("We where unable to find your home directory");
    let filename: String = format!("{}/.rmodulestmp{}", homedir.display(), rstr);
    let path = Path::new(&filename);
    let mut tmpfile: File = File::create(&path).expect("Failed to create temporary file");

    if !is_shell_supported(shell) {
        print_usage(true);
        return;
    }


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

        for cmd in command_list {
            if cmd.starts_with(command) {
                rmod::command(cmd, modulename, modules, shell, &mut tmpfile);
                matches = true;
            }
        }

        if !matches {
            print_usage(false);
        }
    }

    let cmd = format!("rm -f {}\n", path.display());
    tmpfile.write_all(cmd.as_bytes()).expect("Unable to write data");
    println!("source {}", path.display());
    // print 'source tmpfile' or '. tmpfile' to output
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 && (&args[1] == "-h" || &args[1] == "--help") {
        print_usage(false);
        return;
    } else if args.len() >= 3 && (&args[1] == "-h" || &args[1] == "--help") {
        print_usage(false);
        return;
    }

    // parse modules path
    let modules: Vec<String>;
    modules = init_modules_path();

    //println!("{:?}", modules);

    parse_commandline(&args, &modules);
}
