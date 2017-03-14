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

fn usage() {

    let error_msg: &str = "Usage: rmodules <shell> <load|unload|list|purge|available> [module \
                           name]";

    show_warning!("{}", &error_msg);
}

fn run(args: &Vec<String>) {
    let shell: &str = &args[1];
    let command: &str;
    let mut modulename: &str = "";

    if !is_shell_supported(shell) {
        usage();
        crash!(1, "{} is not a supported shell", shell);
    }

    // get install dir
    let mut install_dir: String = env::current_dir().unwrap().to_string_lossy().into_owned();

    match env::var("RMODULES_INSTALL_DIR") {
        Ok(path) => install_dir = path,
        Err(_) => {
            show_warning!("$RMODULES_INSTALL_DIR not found, using {}", install_dir);
        }
    };

    let modules = rmod::get_module_list();
    let modulepaths = rmod::get_module_paths();

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

    let mut tmp_file_path: PathBuf;

    match env::home_dir() {
        Some(path) => tmp_file_path = path,
        None => {
            show_warning!("We were unable to find your home directory, checking if /tmp is an \
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
                    search_path: &modulepaths,
                    shell: shell,
                    tmpfile: &tmpfile,
                    installdir: &install_dir,
                };
                rmod::command(&mut rmod_command);
                matches = true;
            }
        }

        if !matches {
            usage();
        }
    }

    // we want a self destructing tmpfile
    // so it must delete itself at the end of the run
    let cmd = format!("rm -f {}\n", tmp_file_path.display());
    crash_if_err!(1, tmpfile.write_all(cmd.as_bytes()));

    // source tmpfile
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
        usage();
        return;
    } else if args.len() >= 3 && (&args[1] == "-h" || &args[1] == "--help") {
        usage();
        return;
    }

    run(&args);
}


#[cfg(test)]
mod tests {
    use super::is_shell_supported;

    #[test]
	fn supported_shells() {
		assert_eq!(false, is_shell_supported("randomshellname"));
		assert_eq!(true, is_shell_supported("bash"));
		assert_eq!(true, is_shell_supported("zsh"));
		assert_eq!(true, is_shell_supported("tcsh"));
		assert_eq!(true, is_shell_supported("csh"));
	}
}
