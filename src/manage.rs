use std::fs;
use std::io::Write;
use std::path::Path;
use rsmod::{Rsmodule, get_module_paths};
use wizard::{is_yes, read_input_shell};
use std::sync::Mutex;

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

pub fn create(rsmod: &Rsmodule) {}
