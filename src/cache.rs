use std::io::{BufWriter, BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::cmp::Ordering;

use walkdir::WalkDir;
extern crate bincode;
use bincode::rustc_serialize::{encode_into, decode_from};

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
        self.name.cmp(&other.name)
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


/*
fn is_dir(entry: &walkdir::DirEntry) -> bool {
    let str_path: &str = entry.file_name().to_str().unwrap();
    let path: PathBuf = PathBuf::from(str_path);
    path.is_dir()
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
*/

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
        let file = BufReader::new(File::open(module_path).unwrap());
        for (_, line) in file.lines().enumerate() {
            buffer = line.unwrap();
        }
    }

    if buffer == modulename {
        return true;
    }

    return false;
}

// TODO:: check to catch the error when the directory doesn't exist
pub fn update(modulepath: String) {

    // TODO: check if we have read and write permissions on 'modulepath'
    // list is: path to file, module name, default
    let mut list: Vec<(String, String, bool)> = Vec::new();
    let module_path = Path::new(&modulepath);

    for entry in WalkDir::new(module_path).into_iter().filter_map(|e| e.ok()) {

        //let str_path: String = entry.file_name().to_str().unwrap().to_string();
        let str_path: &str = entry.path().to_str().unwrap();

        let part: Vec<&str> = str_path.split(&modulepath).collect();

        if !entry.path().is_dir() {

            for mut modulename in part {
                if modulename != "" {

                    let first = modulename.chars().next().unwrap();
                    // FIXME: this might crash if module name is too short
                    let second = &modulename[1..2];

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

    //println_stderr!("{:?}", list);
    // now we have all the module files in the current folder
    // we need to parse them to get their description

    // our list of modules that we will save into the .modulesindex
    let mut modules: Vec<Module> = vec![];

    for (modulepath, modulename, default) in list {
        let path: PathBuf = PathBuf::from(&modulepath);

        let description: Vec<String> =
            super::get_module_description(&path, &modulename, "description");
        let description = description.join(" ");

        // flags is supposed to be a bitfield
        // currently it is only used for flagging a module as default
        let mut flags: i64 = 0;
        if default {
            flags = 1;
        }
        add_module(modulename, description, flags, &mut modules);
        //println_stderr!("{}", description);
    }

    let mut writer = BufWriter::new(File::create(format!("{}/.modulesindex", modulepath)).unwrap());
    encode_into(&modules, &mut writer, bincode::SizeLimit::Infinite).unwrap();
}

pub fn parse_modules_cache_file(filename: &PathBuf, modules: &mut Vec<String>) {

    let mut reader = BufReader::new(File::open(filename).unwrap());
    let decoded: Vec<Module> = decode_from(&mut reader, bincode::SizeLimit::Infinite).unwrap();

    for module in decoded {
        //let tmp = format!("{}\t{}", module.name, module.description);
        //super::write_av_output(&tmp, &tmpfile);
        modules.push(module.name);
    }
}

pub fn get_module_list(tmpfile: &File, arg: &str, shell: &str) {
    let mut bold_start: &str = "$(tput bold)";
    let mut bold_end: &str = "$(tput sgr0)";

    if shell == "tcsh" || shell == "csh" {
        bold_start = "\\033[1m";
        bold_end = "\\033[0m";
    }

    let modulepaths = super::get_module_paths();

    let mut longest_name = 0;
    let mut decoded: Vec<Module> = Vec::new();
    for modulepath in modulepaths.clone() {
        let mut reader = BufReader::new(File::open(format!("{}/.modulesindex", modulepath))
            .unwrap());
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


    for module in decoded {
        let tmp: String;

        // TODO: get env var $COLUMNS (if bash) and format output according to that width
        // in csh you can use: 'stty size'
        if module.flags == 1 {
            tmp = format!("{}{:width$}{}{}",
                          bold_start,
                          module.name,
                          bold_end,
                          module.description,
                          width = longest_name);
        } else {
            tmp = format!("{:width$}{}",
                          module.name,
                          module.description,
                          width = longest_name);
        }
        if arg != "" {
            let avmodule_lc: String = module.name.to_lowercase();
            let module_lc: String = arg.to_lowercase();
            let avmodule_lc: &str = avmodule_lc.as_ref();
            let module_lc: &str = module_lc.as_ref();

            if avmodule_lc.contains(module_lc) {
                super::write_av_output(&tmp, &tmpfile);
            }
        } else {
            super::write_av_output(&tmp, &tmpfile);
        }
    }
}
