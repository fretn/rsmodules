use std::fs;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;
use std::io;
use std::path::Path;
use std::env;
use rsmod::{Rsmodule, get_module_paths};
use wizard::{is_yes, read_input_shell};
use std::sync::Mutex;

use getopts::{Options, Matches};

lazy_static! {
	static ref REMOVED_MODULES: Mutex<bool> = Mutex::new(false);
}

fn remove_file(filename: &str) {
    let mut err: bool = false;
    fs::remove_file(filename).unwrap_or_else(|why| {
        println_stderr!("Could not remove modulefile: {:?}", why.kind());
        err = true;
    });

    if !err {
        let mut removed = REMOVED_MODULES.lock().unwrap();
        *removed = true;
    }
}

pub fn delete(rsmod: &Rsmodule) {
    let mut interactive: bool = true;

    if rsmod.shell == "noshell" {
        interactive = false;
    }

    let toremove: Vec<&str> = rsmod.arg.split_whitespace().collect();
    for module in toremove.iter() {
        for path in rsmod.search_path.iter() {
            let filename: &str = &format!("{}/{}", path, module);
            if Path::new(filename).is_file() {
                if interactive {
                    if is_yes(read_input_shell(&format!("Are you sure you want to delete the modulefile {} ? [Y/n]: ",
                                                        filename),
                                               rsmod.shell)) {
                        remove_file(filename);
                    } else {
                        println_stderr!("No module files where deleted.");
                    }
                } else {
                    remove_file(filename);
                }
            }
        }
    }

    let removed = REMOVED_MODULES.lock().unwrap();
    if *removed {
        if interactive &&
           is_yes(read_input_shell(&format!("Removal of {} was sucessful.\nDo you want to update the module cache now ? \
                                             [Y/n]: ",
                                            rsmod.arg),
                                   rsmod.shell)) {
            let modulepaths = get_module_paths(false);
            for modulepath in modulepaths {
                if modulepath != "" {
                    super::cache::update(modulepath, rsmod.shell);
                }
            }
        } else {
            println!("Removal of {} was succesful. Don't forget to update the module cache.",
                     rsmod.arg);
        }
    }
}

fn print_usage(opts: Options) {
    let brief = format!("Usage: module create [options]");
    println_stderr!("{}", opts.usage(&brief));
}

pub fn create(rsmod: &Rsmodule) {
    let mut output: Vec<String> = Vec::new();

    if rsmod.shell == "noshell" {
        let mut opts = Options::new();
        let mut option_commands: Vec<(&str, &str, &str, i32, &str, &str)> = Vec::new();
        option_commands.push(("h", "help", "", 0, "help", ""));
        option_commands.push(("f", "filename", "", 10, "output filename", ""));
        option_commands.push(("d", "description", "description", 1, "set a description", "DESCRIPTION"));
        option_commands.push(("p", "prepend-path", "prepend_path", 2, "prepend a path to a variable", "VARNAME,VALUE"));
        option_commands.push(("a", "append-path", "append_path", 2, "append a path to a variable", "VARNAME,VALUE"));
        option_commands.push(("r", "remove-path", "remove_path", 2, "remove a path from a variable", "VARNAME,VALUE"));
        option_commands.push(("s", "setenv", "setenv", 2, "set an environment variable", "VARNAME,VALUE"));
        option_commands.push(("g", "getenv", "getenv", 1, "get an environment variable", "VARNAME"));
        option_commands.push(("u", "unsetenv", "unsetenv", 1, "unset an environment variable", "VARNAME"));
        option_commands.push(("l", "load", "load", 1, "load a module", "MODULENAME"));
        option_commands.push(("U", "unload", "unload", 1, "unload a module", "MODULENAME"));
        option_commands.push(("c", "conflict", "conflict", 1, "conflict with a module", "MODULENAME"));
        option_commands.push(("S", "system", "system", 1, "run a system command", "COMMAND"));
        option_commands.push(("A", "set-alias", "set_alias", 2, "create an alias", "COMMAND"));

        for (short, long, _, number, desc, hint) in option_commands.clone() {
            if number == 0 {
                opts.optflag(short, long, desc);
            } else if number == 10 {
                // nasty
                opts.reqopt(short, long, desc, hint);
            } else {
                opts.optmulti(short, long, desc, hint);
            }
        }

        let args: Vec<String> = env::args().collect();

        let matches = match opts.parse(&args[3..]) {
            Ok(m) => m,
            Err(f) => {
                crash!(super::super::CRASH_CREATE_ERROR, "{}", f.to_string());
            }
        };


        let mut present: Vec<String> = Vec::new();
        for (opt, _, _, _, _, _) in option_commands.clone() {
            present.push(opt.to_string());
        }

        let present: bool = matches.opts_present(&present);
        if !present {
            if rsmod.arg != "" {
                print_usage(opts);
            } else {
                run_create_wizard(rsmod.shell, &mut output, get_modulename(rsmod.arg).as_ref());
            }
            return;
        }

        // help
        if matches.opt_present("h") {
            print_usage(opts);
            return;
        }

        // parse other options
        for (opt, _, command, number, _, _) in option_commands {
            if opt != "h" && opt != "f" {
                parse_opt(&matches, &mut output, opt, command, number);
            }
        }

        // write to file
        if matches.opt_present("f") {
            let filename = matches.opt_str("f").unwrap();
            match save(&filename, &output) {
                Ok(_) => {}
                Err(e) => {
                    println_stderr!("Cannot write to file {} ({})", filename, e);
                    ::std::process::exit(super::super::CRASH_CREATE_ERROR);
                }
            }
        }
    } else {
        run_create_wizard(rsmod.shell, &mut output, get_modulename(rsmod.arg).as_ref());
    }
}

fn get_modulename(arg: &str) -> String {
    let _arg: Vec<&str> = arg.split_whitespace().collect();

    if _arg.len() != 1 {
        println_stderr!("usage: module create [modulename]");
        //super::super::usage(true);
        ::std::process::exit(super::super::CRASH_CREATE_ERROR);
    }

    return arg.to_string();
}

fn save(filename: &str, output: &[String]) -> io::Result<()> {

    if !Path::new(&filename).is_file() {

        let mut path = Path::new(&filename);
        path = path.parent().unwrap();
        create_dir_all(path)?;

        let mut file: File = match OpenOptions::new().write(true).create(true).truncate(true).open(&filename) {
            Ok(fileresult) => fileresult,
            Err(_) => return Err(io::Error::last_os_error()),
        };

        for line in output {
            if writeln!(file, "{}", line).is_err() {
                return Err(io::Error::last_os_error());
            }
        }
        println_stderr!("\nThe creation of modulefile {} was succesful. Don't forget to update the module cache.",
                        filename);
    } else {
        println_stderr!("The file {} already exists, aborting.", filename);
        ::std::process::exit(super::super::CRASH_CREATE_ERROR);
    }

    Ok(())
}

fn parse_opt(matches: &Matches, output: &mut Vec<String>, opt: &str, command: &str, number: i32) {
    if matches.opt_present(opt) {
        let value: Vec<String> = matches.opt_strs(opt);
        for i in &value {
            if number == 1 {
                let msg = format!("{}(\"{}\");", command, i);
                output.push(msg);
            } else if number == 2 {
                let result: Vec<&str> = i.split(',').collect();
                if result.get(0) != None && result.get(1) != None {
                    let msg = format!("{}(\"{}\",\"{}\");", command, &result[0], &result[1]);
                    output.push(msg);
                }
            }
        }
    }
}

pub fn add_description(shell: &str, mut output: &mut Vec<String>, skip: bool, modulename: &str) {
    if !skip {
        let desc = read_input_shell(&format!(" * Enter a description for the module {}: ", modulename),
                                    shell)
            .trim_right_matches('\n')
            .to_string();
        output.push(format!("description(\"{}\");", desc));
    }

    if is_yes(read_input_shell(" * Do you want to add another description entry ? [Y/n]: ",
                               shell)) {
        let desc = read_input_shell("   Enter your description: ", shell).trim_right_matches('\n').to_string();
        output.push(format!("description(\"{}\");", desc));
        add_description(shell, &mut output, true, modulename);
        println_stderr!("");
    }
}

pub fn add_path(shell: &str, mut output: &mut Vec<String>, skip: bool) {
    if !skip {
        let val = read_input_shell("   Enter the path where the executables can be found: ",
                                   shell)
            .trim_right_matches('\n')
            .to_string();
        output.push(format!("prepend_path(\"PATH\",\"{}\");", val));
        if is_yes(read_input_shell(" * Do you want to set the LD_LIBRARY_PATH variable? [Y/n]: ",
                                   shell)) {
            let val = read_input_shell("   Enter the path where the libraries can be found: ",
                                       shell)
                .trim_right_matches('\n')
                .to_string();
            output.push(format!("prepend_path(\"LD_LIBRARY_PATH\",\"{}\");", val));
        }
    }
    println_stderr!("");
    if is_yes(read_input_shell(" * Do you want to set another path variable? [Y/n]: ",
                               shell)) {
        let var = read_input_shell("   Enter the name of variable: ", shell).trim_right_matches('\n').to_string();
        let val = read_input_shell("   Enter the path you want to add: ", shell).trim_right_matches('\n').to_string();
        output.push(format!("prepend_path(\"{}\",\"{}\");", var, val));
        add_path(shell, &mut output, true);
    }
}

pub fn run_create_wizard(shell: &str, mut output: &mut Vec<String>, modulename: &str) {
    println_stderr!("");
    // Where do you want to save this modulefile ?
    // for path in modulepath
    // 1.
    // 2.
    // 3.
    // please enter a number

    // Enter the root directory of the installation

    //
    println_stderr!("");
    add_description(shell, &mut output, false, modulename);
    add_path(shell, &mut output, false);
    for line in output {
        println_stderr!("{}", line);
    }
}
