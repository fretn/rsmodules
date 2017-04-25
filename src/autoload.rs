use std::fs::OpenOptions;
use std::io::{Write, BufRead, BufReader};
use std::path::Path;
use std::fs::File;
use std::sync::Mutex;

extern crate shellexpand;
extern crate regex;

use regex::Regex;

static AUTOLOAD_FILE: &'static str = "~/.rsmodules_autoload";
lazy_static! {
    static ref OUTPUT_BUFFER: Mutex<Vec<String>> = Mutex::new(vec![]);
}

fn echo_output_buffer(shell: &str) {
    let output_buffer = OUTPUT_BUFFER.lock().unwrap();

    for line in output_buffer.iter() {
        super::echo(&format!("  * {}", line), shell);
    }

}

fn empty_output_buffer() {
    let mut output_buffer = OUTPUT_BUFFER.lock().unwrap();
    let ref mut output_buffer = *output_buffer;
    output_buffer.clear();
}

fn list(modulenames: &str) {
    let mut output_buffer = OUTPUT_BUFFER.lock().unwrap();
    let ref mut output_buffer = *output_buffer;

    let modulenames: Vec<&str> = modulenames.split_whitespace().collect();
    for modulename in modulenames.iter() {
        output_buffer.push(modulename.to_string());
    }
}

fn get_module_autoload_string(modules: Vec<&str>, existing: &str, subcommand: &str) -> String {

    let mut output: String = format!("{}", existing);

    if modules.len() == 0 {
        return output;
    }

    if subcommand == "append" || subcommand == "add" {
        output = format!("{} {}", existing, modules.join(" "));
    }

    if subcommand == "prepend" {
        output = format!("{} {}", modules.join(" "), existing);
    }

    if subcommand == "remove" {
        let mut modules_output: Vec<&str> = Vec::new();
        let existing: Vec<&str> = existing.split_whitespace().collect();
        for existing_module in existing.iter() {
            let mut found = false;
            for module in modules.iter() {
                if module == existing_module {
                    found = true;
                }
            }

            if !found {
                modules_output.push(existing_module);
            }
        }

        output = format!("{}", modules_output.join(" "));
    }

    return output;
}

fn is_module_autoloaded(module: &str, existing: &str) -> bool {
    let existing: Vec<&str> = existing.split_whitespace().collect();

    for item in existing.iter() {
        if module == *item {
            return true;
        }
    }

    return false;
}

fn check_init_file(filename: &str) {
    let line: &str = &format!("source {}", AUTOLOAD_FILE);
    // FIXME: errors from detect and append_line are printed to stdout
    if !detect_line(line, filename) {
        append_line(line, filename, false);
    }

}

fn append_line(line: &str, filename: &str, verbose: bool) -> bool {
    return super::super::wizard::append_line(line, filename, verbose);
}

fn detect_line(line: &str, filename: &str) -> bool {
    return super::super::wizard::detect_line(line, filename);
}

pub fn run(subcommand: &str, args: &mut Vec<&str>, shell: &str) {
    // .bashrc (and others)
    // should contain: source .rsmodules
    // .rsmodules then contains the module load commands
    // this is to prevent us from adding to module load inside
    // an if else structure

    // module autoload list should be smart enough
    // to list the modules from .bashrc also, but it will
    // say that these are externally handled

    // detect 'module load' in init script
    // check if it has multiple modules added in one command
    // check if args.get(x) matches with one of them or not
    // if not, add it

    let initfile = match shell {
        "bash" => "~/.bashrc",
        "csh" => "~/.cshrc",
        "zsh" => "~/.zshrc",
        "tcsh" => "~/.tcshrc",
        _ => "~/.login", // meh ?
    };

    let initfile: &str = &shellexpand::tilde(initfile);
    check_init_file(initfile);

    // for line in initfile
    // run regex
    // if match:
    //  if add, add module
    //  if purge, remove line
    //  if list, list modules
    //  if remove, remove module
    // write file

    if subcommand == "list" {
        empty_output_buffer();
        super::echo("", shell);
        super::echo("  Autoloaded modules NOT managed by rsmodules:", shell);
        super::echo("", shell);
        parse_file(subcommand, args, initfile);
        echo_output_buffer(shell);
    }

    if subcommand == "list" {
        empty_output_buffer();
        super::echo("", shell);
        super::echo("  Autoloaded modules managed by rsmodules:", shell);
        super::echo("", shell);
    }
    parse_file(subcommand, args, &shellexpand::tilde(AUTOLOAD_FILE));
    if subcommand == "list" {
        echo_output_buffer(shell);
        super::echo("", shell);
    }

}

fn parse_file(subcommand: &str, args: &mut Vec<&str>, initfile: &str) {
    let mut output: Vec<String> = Vec::new();

    let mut num_matches = 0;
    if Path::new(initfile).is_file() {
        let initfile: File = match File::open(initfile) {
            Ok(initfile) => initfile,
            Err(_) => {
                return;
            }
        };

        let initfile = BufReader::new(initfile);
        let mut done = false;
        for (_, entry) in initfile.lines().enumerate() {
            let buffer = entry.unwrap();
            let re = Regex::new(r#"^\s*(?P<module>module)\s+(?P<subcommand>[a-zA-Z0-9]*)\s+(?P<modules>.*)"#).unwrap();

            if subcommand == "list" {
                for cap in re.captures_iter(&buffer) {
                    if &cap["subcommand"] == "load" {
                        list(&cap["modules"]);
                    }
                    //println_stderr!("'{}' '{}' '{}'", &cap["module"], &cap["subcommand"], &cap["modules"]);
                }
            } else if subcommand == "append" || subcommand == "add" || subcommand == "prepend" ||
                      subcommand == "remove" {
                let mut matched = false;
                if !done {
                    for cap in re.captures_iter(&buffer) {
                        if &cap["subcommand"] == "load" {
                            let mut modules: Vec<&str> = Vec::new();
                            for module in args.iter() {
                                if subcommand == "append" || subcommand == "add" || subcommand == "prepend" {
                                    if !is_module_autoloaded(module, &cap["modules"]) {
                                        modules.push(module);
                                    }
                                } else if subcommand == "remove" {
                                    if is_module_autoloaded(module, &cap["modules"]) {
                                        modules.push(module);
                                    }
                                }
                            }

                            if subcommand == "purge" {
                                done = true;
                            } else {
                                if modules.len() > 0 {
                                    let module_list: String =
                                        get_module_autoload_string(modules, &cap["modules"], subcommand);
                                    output.push(format!("module load {}", module_list));
                                    done = true;
                                }
                            }
                        }
                        matched = true;
                        num_matches += 1;
                    }
                }

                if !matched {
                    output.push(buffer);
                }
            }
        }
    } else {
        // when the file doesn't exist, just add the module load command
        if subcommand == "append" || subcommand == "add" || subcommand == "prepend" {
            output.push(format!("module load {}", args.join(" ")));
        }
    }

    //  when the file is empty, just add the module load command
    if num_matches == 0 && Path::new(initfile).is_file() {
        if subcommand == "append" || subcommand == "add" || subcommand == "prepend" {
            output.push(format!("module load {}", args.join(" ")));
        }
    }

    // write to the file ~/.rsmodules_autoload
    if subcommand == "append" || subcommand == "add" || subcommand == "prepend" || subcommand == "remove" ||
       subcommand == "purge" {
        //if output.len() > 0 && initfile == &shellexpand::tilde(AUTOLOAD_FILE) {
        if initfile == &shellexpand::tilde(AUTOLOAD_FILE) {
            let mut file: File = match OpenOptions::new().write(true).create(true).truncate(true).open(initfile) {
                Ok(fileresult) => fileresult,
                Err(e) => {
                    println_stderr!("Cannot write to file {} ({})", initfile, e);
                    return;
                }
            };

            for newline in output {
                if let Err(e) = writeln!(file, "{}", newline) {
                    super::crash(super::super::CRASH_CANNOT_ADD_TO_ENV,
                                 &format!("Cannot write to file {} ({})", initfile, e));
                }
            }
        }
    }
}
