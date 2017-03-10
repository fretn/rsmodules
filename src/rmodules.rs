use std::fs::File;
use std::io::Write;

mod rmodule;
use rmodule::Rmodule;

pub fn command(rmodule: Rmodule) {
/*
pub fn command(command: &str,
               module: &str,
               modules: &Vec<String>,
               shell: &str,
               tmpfile: &mut File) {
               */
    //    println!("{} {} {:?} {}", command, module, modules, shell);

    if rmodule.command == "load" {
        load(rmodule.module.as_ref(), rmodule.shell.as_ref());
    } else if rmodule.command == "unload" {
        unload(rmodule.module.as_ref(), rmodule.shell.as_ref());
    } else if rmodule.command == "available" {
        available(rmodule.module.as_ref(), rmodule.modules.as_ref(), &rmodule.tmpfile);
    } else if rmodule.command == "list" {
        list(rmodule.module.as_ref(), rmodule.shell.as_ref());
    } else if rmodule.command == "purge" {
        purge(rmodule.module.as_ref(), rmodule.shell.as_ref());
    }
}

fn load(module: &str, shell: &str) {
    println!("load {} {}", module, shell);

    // check if module file exists

    // if not, maybe check if its a partial match
    // blast -> blast/x86_64/1.0 and blast/x86_64/2.0
    // then we need to load the Default version
    // or just the latest one
}

fn unload(module: &str, shell: &str) {
    println!("unload {} {}", module, shell);
}

fn available(module: &str, modules: &Vec<String>, mut tmpfile: &File) {

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
                write_av_output(&avmodule, &mut tmpfile);
            }
        } else {
            write_av_output(&avmodule, &mut tmpfile);
        }
    }
}

fn write_av_output(line: &str, mut tmpfile: &File) {
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
