extern crate rand;
use rand::Rng;

#[path = "rmodules.rs"]
mod rmod;

use rmod::Rmodule;

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

fn init_modules() -> Vec<String> {
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

fn run_commandline_args(args: &Vec<String>, modules: Vec<String>) {
    let shell: &str = &args[1];
    let command: &str;
    let mut modulename: &str = "";

    if !is_shell_supported(shell) {
        print_usage(true);
        return;
    }

    // create temporary file in the home folder
    // if the file cannot be created try to create it
    // in /tmp, if that fails, the program exits
    //
    // ~/.rmodulestmpXXXXXXXX
    // /tmp/.rmodulestmpXXXXXXXX

    let mut tmpfile: File;

    let rstr: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(8)
        .collect();

    //let mut tmp_file_path: PathBuf = env::home_dir()
    //    .expect("We were unable to find your home directory");

    let mut tmp_file_path: PathBuf;

    match env::home_dir() {
        Some(path) => tmp_file_path = path,
        None => {
            println_stderr!("We were unable to find your home directory, checking if /tmp is an \
                             option");

            // this is wrong, as we try to use temp again a bit later
            tmp_file_path = env::temp_dir();
            // return;
        }
    };

    let filename: String = format!(".rmodulestmp{}", rstr);
    let filename: &str = filename.as_ref();
    tmp_file_path.push(filename);

    match File::create(&tmp_file_path) {
        Ok(file) => tmpfile = file,
        Err(_) => {
            // home exists but we can't create the temp file in it or
            // worst case, /tmp exists but we can't create the temp file in it
            tmp_file_path = env::temp_dir();
            let filename: String = format!(".rmodulestmp{}", rstr);
            let filename: &str = filename.as_ref();
            tmp_file_path.push(filename);

            match File::create(&tmp_file_path) {
                Ok(newfile) => tmpfile = newfile,
                Err(e) => {
                    println_stderr!("Failed to create temporary file: {}", e);
                    return;
                }
            };
        }
    };

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
                let rmodule: Rmodule = Rmodule { command: cmd, module: modulename, modules: &modules, shell: shell, tmpfile: &mut tmpfile };
                //rmod::command(cmd, modulename, modules, shell, &mut tmpfile);
                rmod::command(rmodule);
                matches = true;
            }
        }

        if !matches {
            print_usage(false);
        }
    }

    let cmd = format!("rm -f {}\n", tmp_file_path.display());

    // TODO: use a match to catch this error
    tmpfile.write_all(cmd.as_bytes()).expect("Unable to write data");

    if shell == "tcsh" || shell == "csh" {
        println!(". {}", tmp_file_path.display());
    } else {
        println!("source {}", tmp_file_path.display());
    }
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
    modules = init_modules();

    //println!("{:?}", modules);

    run_commandline_args(&args, modules);
}
