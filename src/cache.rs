/*
MIT License

Copyright (c) 2017 Frederik Delaere

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
use std::io::{BufWriter, BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::cmp::Ordering;

use walkdir::WalkDir;
extern crate bincode;
use bincode::rustc_serialize::{encode_into, decode_from};

pub static MODULESINDEX: &'static str = ".modulesindex";

#[derive(RustcEncodable, RustcDecodable, Clone, Eq)]
struct Module {
    name: String,
    description: String,
    flags: i64,
}

impl Module {
    pub fn new() -> Module {
        Module {
            name: String::new(),
            description: String::new(),
            flags: 0,
        }
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Module) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Module) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Module) -> bool {
        self.name == other.name
    }
}

fn add_module(name: String, description: String, flags: i64, modules: &mut Vec<Module>) {
    let mut module: Module = Module::new();
    module.name = name;
    module.description = description;
    module.flags = flags;

    modules.push(module);
}


fn get_default_version(modulepath: &str, modulename: &str) -> bool {
    let parts: Vec<&str> = modulename.split('/').collect();
    let mut groupname: &str = "";
    if parts.len() >= 1 {
        groupname = parts[0];
    }
    let tmp = format!("{}/{}/.version", modulepath, groupname);
    let module_path = Path::new(&tmp);

    // read filename line by line, and push it to modules
    let mut buffer = String::new();

    if Path::new(&module_path).is_file() {
        let file: File = match File::open(module_path) {
            Ok(file) => file,
            Err(_) => {
                return false;
            }
        };

        let file = BufReader::new(file);
        for (_, line) in file.lines().enumerate() {
            buffer = line.unwrap();
        }
    }

    if buffer == modulename {
        return true;
    }

    return false;
}

pub fn update(modulepath: String, shell: &str) {

    // TODO: check if we have read and write permissions on 'modulepath'

    // list is: path to file, module name, default
    let mut list: Vec<(String, String, bool)> = Vec::new();
    let module_path = Path::new(&modulepath);

    for entry in WalkDir::new(module_path).into_iter().filter_map(|e| e.ok()) {

        let str_path: &str = entry.path().to_str().unwrap();

        let part: Vec<&str> = str_path.split(&modulepath).collect();

        if !entry.path().is_dir() {

            for mut modulename in part {
                if modulename != "" {

                    let first = modulename.chars().next().unwrap();
                    let mut second = "";

                    if modulename.len() >= 2 {
                        second = &modulename[1..2];
                    }

                    if modulename == MODULESINDEX {
                        continue;
                    }
                    // modulename can start with /
                    // also we don't want the .modulesindex or other hidden files
                    if first == '/' && second != "." {
                        modulename = modulename.trim_left_matches('/');
                    }

                    let modulename_part: Vec<&str> = modulename.split('/').collect();
                    let mut is_version_file = false;

                    // skip the .version files, these are not modulefiles
                    for mp in modulename_part {
                        if mp == ".version" {
                            is_version_file = true;
                        }
                    }

                    if second != "." && !is_version_file {
                        let default = get_default_version(&modulepath, modulename);
                        list.push((str_path.to_string(), modulename.to_string(), default));
                    }
                }
            }
        }
    }

    // now we have all the module files in the current folder
    // we need to parse them to get their description

    // our list of modules that we will save into the .modulesindex
    let mut modules: Vec<Module> = vec![];

    for (modulepath, modulename, default) in list {
        let path: PathBuf = PathBuf::from(&modulepath);

        let description: Vec<String> = super::get_module_description(&path, "description");
        let description = description.join(" ");

        // flags is supposed to be a bitfield
        // currently it is only used for flagging a module as default
        let mut flags: i64 = 0;
        if default {
            flags = 1;
        }
        add_module(modulename, description, flags, &mut modules);
    }

    let file_str = format!("{}/{}", modulepath, MODULESINDEX);
    let file: File = match File::create(&file_str) {
        Ok(file) => file,
        Err(_) => {
            show_warning!("Something went wrong while trying to update: {}", &file_str);
            return;
        }
    };

    let mut writer = BufWriter::new(file);
    encode_into(&modules, &mut writer, bincode::SizeLimit::Infinite).unwrap();

    let msg: String = format!("The index file {} was succesfully updated.", &file_str);

    super::echo(&msg, shell);
}

pub fn parse_modules_cache_file(filename: &PathBuf, modules: &mut Vec<String>) {

    let file: File = match File::open(filename) {
        Ok(file) => file,
        Err(_) => {
            super::crash(super::super::CRASH_COULDNT_OPEN_CACHE_FILE,
                         "modules_cache_file: couldn't open the required index file");
            return;
        }
    };
    let mut reader = BufReader::new(file);
    let decoded: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite).unwrap();

    for module in decoded {
        modules.push(module.name);
    }
}

pub fn get_module_list(arg: &str, shell: &str, shell_width: usize) {
    let mut bold_start: &str = "$(tput bold)";
    let mut bold_end: &str = "$(tput sgr0)";

    if shell == "tcsh" || shell == "csh" {
        bold_start = "\\033[1m";
        bold_end = "\\033[0m";
    }

    let modulepaths = super::get_module_paths(false);

    let mut simple_list: bool = false;

    // prints a nice list for module av
    // no gaps, no default, no description
    // usefull for parsing, eg for bash completion
    if shell == "noshell" || shell == "python" {
        simple_list = true;
    }

    let mut longest_name = 0;
    let mut decoded: Vec<Module> = Vec::new();
    for modulepath in modulepaths.clone() {

        let file: File = match File::open(format!("{}/{}", modulepath, MODULESINDEX)) {
            Ok(file) => file,
            Err(_) => {
                continue;
            }

        };

        let mut reader = BufReader::new(file);
        let decoded_file: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite)
            .unwrap();
        for item in decoded_file {
            decoded.push(item);
        }

        for module in decoded.clone() {
            if longest_name <= module.name.len() {
                longest_name = module.name.len();
            }
        }
        longest_name = longest_name + 1;
    }

    decoded.sort();

    let mut previous_first_char: char = '§';
    for module in decoded {
        let tmp: String;

        let mut description = module.description;
        description.truncate(shell_width - longest_name - 3);

        let mut default: &str = " ";

        if module.flags == 1 {
            default = "D";
        }

        if simple_list {
            tmp = format!("{}", module.name);
        } else {

            // print loaded modules in bold
            if super::is_module_loaded(module.name.as_ref()) {

                tmp = format!("{} {}{:width$}{}{}",
                              default,
                              bold_start,
                              module.name,
                              bold_end,
                              description,
                              width = longest_name);
            } else {
                tmp = format!("{} {:width$}{}",
                              default,
                              module.name,
                              description,
                              width = longest_name);
            }
        }


        if arg != "" {
            let avmodule_lc: String = module.name.to_lowercase();
            let module_lc: String = arg.to_lowercase();
            let avmodule_lc: &str = avmodule_lc.as_ref();
            let module_lc: &str = module_lc.as_ref();

            if avmodule_lc.contains(module_lc) {
                let first_char: char = module.name.chars().next().unwrap();
                if first_char != previous_first_char && !simple_list {
                    // add a newline
                    super::echo("", shell);
                }
                previous_first_char = module.name.chars().next().unwrap();
                if simple_list {
                    println!("{}", &tmp);
                } else {
                    super::echo(&tmp, shell);
                }
            }
        } else {
            let first_char: char = module.name.to_lowercase().chars().next().unwrap();
            if first_char != previous_first_char && !simple_list {
                // add a newline
                super::echo("", shell);
            }
            previous_first_char = module.name.to_lowercase().chars().next().unwrap();
            if simple_list {
                println!("{}", &tmp);
            } else {
                super::echo(&tmp, shell);
            }
        }
    }
}
