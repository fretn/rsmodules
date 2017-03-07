extern crate tempfile;

#[path = "rmodules.rs"]
mod rmod;

use std::io::Write;
use std::fs::File;

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

    let shell: &str = &args[1];

    if !is_shell_supported(shell) {
        print_usage(true, true);
        return;
    }

    let command: &str;
    let modulename: &str;

    if args.len() >= 3 {
        command = &args[2];

        if command == "load" || command == "unload" || command == "available" {
            if args.len() > 3 {
                modulename = &args[3];
                rmod::load(modulename);
                //run_command(command, modulename);
            } else {
                print_usage(false, true);
                return;
            }
        } else if command == "list" || command == "purge" {
            //run_command(command);
        } else {
            print_usage(false, true);
            return;
        }
    }
}
