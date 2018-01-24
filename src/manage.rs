use std::fs;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;
use std::io;
use std::path::Path;
use std::env::args;
use rsmod::{get_module_paths, Rsmodule};
use wizard::{is_yes, read_input_shell};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

//use getopts::{Options, Matches};

use gumdrop::Options;

lazy_static! {
    static ref REMOVED_MODULES: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

fn remove_file(filename: &str) {
    let mut err: bool = false;
    fs::remove_file(filename).unwrap_or_else(|why| {
        println_stderr!("Could not remove modulefile: {:?}", why.kind());
        err = true;
    });

    if !err {
        REMOVED_MODULES.store(true, Ordering::Relaxed);
    }
}

pub fn delete(rsmod: &Rsmodule) {
    let interactive = !(rsmod.shell == "noshell");

    let toremove: Vec<&str> = rsmod.arg.split_whitespace().collect();
    for module in &toremove {
        for path in rsmod.search_path.iter() {
            let filename: &str = &format!("{}/{}", path, module);
            if Path::new(filename).is_file() {
                if interactive {
                    if is_yes(&read_input_shell(
                        &format!(
                            "Are you sure you want to delete the modulefile {} ? [Y/n]: ",
                            filename
                        ),
                        rsmod.shell,
                    )) {
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

    if REMOVED_MODULES.load(Ordering::Relaxed) {
        if interactive
            && is_yes(&read_input_shell(
                &format!(
                    "Removal of {} was sucessful.\nDo you want to update the module cache now ? \
                     [Y/n]: ",
                    rsmod.arg
                ),
                rsmod.shell,
            )) {
            let modulepaths = get_module_paths(false);
            for modulepath in modulepaths {
                if modulepath != "" {
                    super::cache::update(&modulepath, rsmod.shell);
                }
            }
        } else {
            println!(
                "Removal of {} was succesful. Don't forget to update the module cache.",
                rsmod.arg
            );
        }
    }
}

/*
fn print_usage(opts: &Options) {
    let brief = "Usage: module create [options]";
    println_stderr!("{}", opts.usage(brief));
}
*/

#[derive(Debug, Default, Options)]
struct CreateOptions {
    // Contains "free" arguments -- those that are not options.
    // If no `free` field is declared, free arguments will result in an error.
    #[options(free)] free: Vec<String>,

    // Boolean options are treated as flags, taking no additional values.
    // The optional `help` attribute is displayed in `usage` text.
    #[options(help = "Print this help message")] help: bool,

    #[options(no_short, help = "Output filename")] filename: Option<String>,

    #[options(no_short, help = "Prepend a path to a variable")] prepend_path: Vec<(String, String)>,

    #[options(no_short, help = "Append a path to a variable")] append_path: Vec<(String, String)>,

    #[options(no_short, help = "Remove a path from a variable")] remove_path: Vec<(String, String)>,

    #[options(no_short, help = "Set a variable")] setenv: Vec<(String, String)>,

    #[options(no_short, help = "Get a variable")] getenv: Vec<String>,

    #[options(no_short, help = "Unset a variable")] unsetenv: Vec<String>,

    #[options(no_short, help = "A description for the module file")] description: Vec<String>,

    #[options(no_short, help = "Load a module")] load: Vec<String>,

    #[options(no_short, help = "Unload a module")] unload: Vec<String>,

    #[options(no_short, help = "Conflict with a module")] conflict: Vec<String>,

    #[options(no_short, help = "Run a system command")] system: Vec<String>,

    #[options(no_short, help = "Create an alias")] set_alias: Vec<String>,
}

fn print_help(args: &[String]) {
    //let args: Vec<String> = args().collect();

    // Printing usage text for the `--help` option is handled explicitly
    // by the program.
    // However, `derive(Options)` does generate information about all
    // defined options.
    println_stderr!("Usage: {} [OPTIONS] [ARGUMENTS]", args[0]);
    println_stderr!("");
    println_stderr!("{}", CreateOptions::usage());
}

fn prepare_for_saving(filename: &str, output: &[String]) {
    match save(filename, output) {
        Ok(_) => {}
        Err(e) => {
            println_stderr!("Cannot write to file {} ({})", filename, e);
            ::std::process::exit(super::super::CRASH_CREATE_ERROR);
        }
    }
}

pub fn create(rsmod: &Rsmodule) {
    let mut output: Vec<String> = Vec::new();
    let args: Vec<String> = args().collect();

    // Remember to skip the first argument. That's the program name.
    let opts = match CreateOptions::parse_args_default(&args[3..]) {
        Ok(opts) => opts,
        Err(e) => {
            println_stderr!("{}: {}", args[0], e);
            return;
        }
    };

    if opts.help {
        print_help(&args);
    } else if rsmod.arg == "" {
        let filename = run_create_wizard(rsmod.shell, &mut output);
        prepare_for_saving(&filename, &output);
    } else if opts.filename == None {
        print_help(&args);
        println_stderr!("");
        println_stderr!("Error:");
        println_stderr!("");
        // TODO: maybe we should just print to stdout when --filename is None
        println_stderr!("  --filename is required");
        println_stderr!("");
    } else {
        println_stderr!("{:#?}", opts);
        prepare_for_saving(&opts.filename.unwrap(), &output);
    }
}
/*
pub fn _create(rsmod: &Rsmodule) {
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
                print_usage(&opts);
            } else {
                run_create_wizard(rsmod.shell, &mut output, get_modulename(rsmod.arg).as_ref());
            }
            return;
        }

        // help
        if matches.opt_present("h") {
            print_usage(&opts);
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

    arg.to_string()
}
*/

fn save(filename: &str, output: &[String]) -> io::Result<()> {
    if !Path::new(&filename).is_file() {
        let mut path = Path::new(&filename);
        path = path.parent().unwrap();
        create_dir_all(path)?;

        let mut file: File = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&filename)
        {
            Ok(fileresult) => fileresult,
            Err(_) => return Err(io::Error::last_os_error()),
        };

        for line in output {
            if writeln!(file, "{}", line).is_err() {
                return Err(io::Error::last_os_error());
            }
        }
        println_stderr!(
            "\nThe creation of modulefile {} was succesful. Don't forget to update the module cache.",
            filename
        );
    } else {
        println_stderr!("The file {} already exists, aborting.", filename);
        ::std::process::exit(super::super::CRASH_CREATE_ERROR);
    }

    Ok(())
}

/*
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
*/

pub fn add_description(shell: &str, mut output: &mut Vec<String>, skip: bool, modulename: &str) {
    if !skip {
        let desc = read_input_shell(
            &format!(" * Enter a description for the module {}: ", modulename),
            shell,
        ).trim_right_matches('\n')
            .to_string();
        output.push(format!("description(\"{}\");", desc));
    }

    if is_yes(&read_input_shell(
        " * Do you want to add another description entry ? [Y/n]: ",
        shell,
    )) {
        let desc = read_input_shell("   Enter your description: ", shell)
            .trim_right_matches('\n')
            .to_string();
        output.push(format!("description(\"{}\");", desc));
        add_description(shell, &mut output, true, modulename);
        println_stderr!("");
    }
}

pub fn add_path(shell: &str, mut output: &mut Vec<String>, skip: bool) {
    if !skip {
        let val = read_input_shell(
            "   Enter the path where the executables can be found: ",
            shell,
        ).trim_right_matches('\n')
            .to_string();
        output.push(format!("prepend_path(\"PATH\",\"{}\");", val));
        if is_yes(&read_input_shell(
            " * Do you want to set the LD_LIBRARY_PATH variable? [Y/n]: ",
            shell,
        )) {
            let val = read_input_shell(
                "   Enter the path where the libraries can be found: ",
                shell,
            ).trim_right_matches('\n')
                .to_string();
            output.push(format!("prepend_path(\"LD_LIBRARY_PATH\",\"{}\");", val));
        }
    }
    println_stderr!("");
    if is_yes(&read_input_shell(
        " * Do you want to set another path variable? [Y/n]: ",
        shell,
    )) {
        let var = read_input_shell("   Enter the name of variable: ", shell)
            .trim_right_matches('\n')
            .to_string();
        let val = read_input_shell("   Enter the path you want to add: ", shell)
            .trim_right_matches('\n')
            .to_string();
        output.push(format!("prepend_path(\"{}\",\"{}\");", var, val));
        add_path(shell, &mut output, true);
    }
}

pub fn run_create_wizard(shell: &str, mut output: &mut Vec<String>) -> String {
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

    // todo: tabcompletion
    // https://github.com/shaleh/rust-readline/blob/master/examples/fileman.rs
    let folder = read_input_shell(
        " * Enter the folder where the modulefile will be saved: ",
        shell,
    ).trim_right_matches('\n')
        .to_string();

    let modulename = read_input_shell(" * Enter the name of the module: ", shell)
        .trim_right_matches('\n')
        .to_string();

    add_description(shell, &mut output, false, &modulename);
    add_path(shell, &mut output, false);
    for line in output {
        println_stderr!("{}", line);
    }

    format!("{}/{}", folder, modulename)
}
