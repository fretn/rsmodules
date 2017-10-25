use std::fs::OpenOptions;
use std::io::{Write, BufRead, BufReader};
use std::path::Path;
use std::fs::File;
use super::{echo, output};
use std::cmp::Ordering;

extern crate shellexpand;
extern crate regex;

use regex::Regex;

#[derive(Clone, Eq, Debug)]
struct Module {
    name: String,
    path: String,
}

impl Module {
    pub fn new() -> Module {
        Module {
            name: String::new(),
            path: String::new(),
        }
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Module) -> Ordering {
        self.path.to_lowercase().cmp(&other.path.to_lowercase())
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Module) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Module) -> bool {
        self.path == other.path
    }
}

static AUTOLOAD_FILE: &str = "~/.rsmodules_autoload";
lazy_static! {
    // Avoid compiling the same regex in a loop
    static ref RE: Regex = Regex::new(r#"^\s*(?P<module>module)\s+(?P<subcommand>[a-zA-Z0-9]*)\s+(?P<modules>.*)"#).unwrap();
    static ref RE_SOURCE: Regex = Regex::new(r#"^\s*(?P<source>\.|source)\s+(?P<path>.*)"#).unwrap();
}

fn get_module_autoload_string(modules: &[&str], existing: &str, subcommand: &str) -> String {

    let mut output: String = existing.to_string();

    if modules.is_empty() {
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
        for existing_module in &existing {
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

        output = modules_output.join(" ");
    }

    output
}

fn is_module_autoloaded(module: &str, existing: &str) -> bool {
    let existing: Vec<&str> = existing.split_whitespace().collect();

    for item in &existing {
        if module == *item {
            return true;
        }
    }

    false
}


fn create_autoload_file() {
    let filename: &str = &shellexpand::tilde(AUTOLOAD_FILE);
    if !Path::new(filename).is_file() {
        match OpenOptions::new().write(true).create_new(true).open(filename) {
            Ok(fileresult) => fileresult,
            Err(e) => {
                println_stderr!("Cannot create  file {} ({})", filename, e);
                return;
            }
        };
    }

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

    let mut bs: &str = "$(tput bold)";
    let mut be: &str = "$(tput sgr0)";

    if shell == "tcsh" || shell == "csh" {
        bs = "\\033[1m";
        be = "\\033[0m";
    }

    // al = autoload
    let mut al_modules: Vec<Module> = Vec::new();

    let initfile = match shell {
        "bash" => "~/.bashrc",
        "csh" => "~/.cshrc",
        "zsh" => "~/.zshrc",
        "tcsh" => "~/.tcshrc",
        _ => "~/.login", // meh ?
    };

    let initfile: &str = &shellexpand::tilde(initfile);

    create_autoload_file();

    // for line in initfile
    // run regex
    // if match:
    //  if add, add module
    //  if purge, remove line
    //  if list, list modules
    //  if remove, remove module
    // write file

    if subcommand == "list" || subcommand == "refurbish" {
        parse_file(subcommand, args, initfile, &mut al_modules);
        if initfile != shellexpand::tilde("~/.login") && (shell == "csh" || shell == "tcsh") {
            let initfile: &str = &shellexpand::tilde("~/.login");
            parse_file(subcommand, args, initfile, &mut al_modules);
        }
    }

    parse_file(subcommand,
               args,
               &shellexpand::tilde(AUTOLOAD_FILE),
               &mut al_modules);

    if subcommand == "refurbish" {

        for al_module in &al_modules {
            output(format!("module load {}\n", al_module.name));
        }
    } else if subcommand == "list" {
        al_modules.sort();
        let mut old_path: String = String::new();
        let mut count = 0;
        for al_module in &al_modules {
            if al_module.path.clone() != shellexpand::tilde(AUTOLOAD_FILE) {
                count += 1;
            }
        }

        if count != 0 && shell != "noshell" {
            echo("", shell);
            echo("  Autoloaded modules NOT managed by RSModules:", shell);
        }
        count = 0;

        for al_module in &al_modules {
            let path = al_module.path.clone();
            if path != shellexpand::tilde(AUTOLOAD_FILE) {
                if path != old_path && shell != "noshell" {
                    echo("", shell);
                    echo(&format!("  Found in: {}", path), shell);
                    echo("", shell);
                }
                if shell == "noshell" {
                    echo(&al_module.name, shell);
                } else {
                    echo(&format!("  * {}{}{}", bs, al_module.name, be), shell);
                }
                old_path = path;
            } else {
                count += 1;
            }
        }

        if count != 0 && shell != "noshell" {
            echo("", shell);
            echo("  Autoloaded modules managed by RSModules:", shell);
            echo("", shell);
        }
        for al_module in &al_modules {
            let path = al_module.path.clone();
            if path == shellexpand::tilde(AUTOLOAD_FILE) {
                if shell == "noshell" {
                    echo(&al_module.name, shell);
                } else {
                    echo(&format!("  * {}{}{}", bs, al_module.name, be), shell);
                }
            }
        }

        if al_modules.is_empty() && shell != "noshell" {
            echo("", shell);
            echo("  No modules are autoloaded.", shell);
        }
        if shell != "noshell" {
            echo("", shell);
        }
    }
}

fn parse_file(subcommand: &str, args: &mut Vec<&str>, initfile: &str, mut al_modules: &mut Vec<Module>) {
    let mut output: Vec<String> = Vec::new();

    let mut num_matches = 0;
    if Path::new(initfile).is_file() {
        let init_file: File = match File::open(initfile) {
            Ok(initfile) => initfile,
            Err(_) => {
                return;
            }
        };

        let initfile_contents = BufReader::new(init_file);
        let mut done = false;
        for (_, entry) in initfile_contents.lines().enumerate() {
            let buffer = entry.unwrap();

            if RE_SOURCE.is_match(&buffer) {
                for cap in RE_SOURCE.captures_iter(&buffer) {
                    let source = &cap["path"];
                    let source: &str = &shellexpand::tilde(source);
                    let source_file_name = Path::new(source).file_name().unwrap();

                    if source_file_name != ".rsmodules_autoload" {
                        parse_file(subcommand, args, source, &mut al_modules);
                    }
                }
            }

            if subcommand == "list" || subcommand == "refurbish" {

                for cap in RE.captures_iter(&buffer) {
                    if &cap["subcommand"] == "load" {

                        let modulenames: &str = &cap["modules"];
                        let modulenames: Vec<&str> = modulenames.split_whitespace().collect();
                        for modulename in &modulenames {
                            let mut al_module: Module = Module::new();
                            al_module.name = modulename.to_string();
                            al_module.path = initfile.to_string();
                            al_modules.push(al_module);
                        }

                    }
                    //println_stderr!("'{}' '{}' '{}'", &cap["module"], &cap["subcommand"], &cap["modules"]);
                }
            } else if subcommand == "append" || subcommand == "add" || subcommand == "prepend" || subcommand == "remove" {
                let mut matched = false;
                if !done {
                    for cap in RE.captures_iter(&buffer) {
                        if &cap["subcommand"] == "load" {
                            let mut modules: Vec<&str> = Vec::new();
                            for module in args.iter() {
                                if subcommand == "append" || subcommand == "add" || subcommand == "prepend" {
                                    if !is_module_autoloaded(module, &cap["modules"]) {
                                        modules.push(module);
                                    }
                                } else if subcommand == "remove" && is_module_autoloaded(module, &cap["modules"]) {
                                    modules.push(module);
                                }
                            }

                            if subcommand == "purge" {
                                done = true;
                            } else {
                                //if modules.len() > 0 {
                                let module_list: String = get_module_autoload_string(&modules, &cap["modules"], subcommand);
                                if !module_list.is_empty() {
                                    output.push(format!("module load {}", module_list));
                                }
                                done = true;
                                //}
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
    if num_matches == 0 && Path::new(initfile).is_file() &&
       (subcommand == "append" || subcommand == "add" || subcommand == "prepend") {
        output.push(format!("module load {}", args.join(" ")));
    }

    // write to the file ~/.rsmodules_autoload
    if (subcommand == "append" || subcommand == "add" || subcommand == "prepend" || subcommand == "remove" ||
        subcommand == "purge") && initfile == shellexpand::tilde(AUTOLOAD_FILE) {
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
