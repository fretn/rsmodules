extern crate rand;
use rand::Rng;

#[macro_use]
mod macros;

#[path = "rmodules.rs"]
mod rmod;

use rmod::Rmodule;

use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::env;

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

fn run_commandline_args(args: &Vec<String>, modules: Vec<String>, modulepaths: Vec<String>) {
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
                    crash!(1, "Failed to create temporary file: {}", e);
                    //return;
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
                let mut rmod_command: Rmodule = Rmodule {
                    cmd: cmd,
                    arg: modulename,
                    list: &modules,
                    paths: &modulepaths,
                    shell: shell,
                    tmpfile: &tmpfile,
                };
                rmod::command(&mut rmod_command);
                matches = true;
            }
        }

        if !matches {
            print_usage(false);
        }
    }

    let cmd = format!("rm -f {}\n", tmp_file_path.display());

    // TODO: use a match to catch this error
    //tmpfile.write_all(cmd.as_bytes()).expect("Unable to write data");
    crash_if_err!(1, tmpfile.write_all(cmd.as_bytes()));

    if shell == "tcsh" || shell == "csh" {
        println!(". {}", tmp_file_path.display());
    } else {
        println!("source {}", tmp_file_path.display());
    }
}

fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        crash!(1, "Try '{0} --help' for more information.", executable!());
    }

    if args.len() >= 2 && (&args[1] == "-h" || &args[1] == "--help") {
        print_usage(false);
        return;
    } else if args.len() >= 3 && (&args[1] == "-h" || &args[1] == "--help") {
        print_usage(false);
        return;
    }

    // parse modules path
    //    let modules: Vec<String>;
    //   let modulepaths: Vec<String>;
    let (modules, modulepaths) = rmod::get_module_list();

    //println!("{:?}", modules);

    run_commandline_args(&args, modules, modulepaths);
}
