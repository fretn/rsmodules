use std::fs::File;
use std::io::Write;


pub fn command(command: &str,
               module: &str,
               modules: &Vec<String>,
               shell: &str,
               tmpfile: &mut File) {
    //    println!("{} {} {:?} {}", command, module, modules, shell);

    if command == "load" {
        load(module, shell);
    } else if command == "unload" {
        unload(module, shell);
    } else if command == "available" {
        available(module, modules, tmpfile);
    } else if command == "list" {
        list(module, shell);
    } else if command == "purge" {
        purge(module, shell);
    }
}

fn load(module: &str, shell: &str) {
    println!("load {} {}", module, shell);
}

fn unload(module: &str, shell: &str) {
    println!("unload {} {}", module, shell);
}

fn available(module: &str, modules: &Vec<String>, tmpfile: &mut File) {

    for avmodule in modules {
        if module != "" {
            let avmodule_lc: String = avmodule.to_lowercase();
            let module_lc: String = module.to_lowercase();
            let avmodule_lc: &str = avmodule_lc.as_ref();
            let module_lc: &str = module_lc.as_ref();

            // contains is case sensitive, lowercase
            // everything
            // TODO: colored output
            if avmodule_lc.contains(module_lc) {
                write_av_output(&avmodule, tmpfile);
            }
        } else {
            write_av_output(&avmodule, tmpfile);
        }
    }
}

fn write_av_output(line: &str, tmpfile: &mut File) {
    let data = format!("echo '{}'\n", line);
    tmpfile.write_all(data.as_bytes()).expect("Unable to write data");
    tmpfile.write_all("\n".as_bytes()).expect("Unable to write data");
}

fn list(module: &str, shell: &str) {
    println!("list {} {}", module, shell);
}

fn purge(module: &str, shell: &str) {
    println!("purge {} {}", module, shell);
}
