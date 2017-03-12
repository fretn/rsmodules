use std::fs::File;
use std::io::Write;

pub struct Rmodule<'a> {
    pub command: &'a str,
    pub module: &'a str,
    pub modules: &'a Vec<String>,
    pub shell: &'a str,
    pub tmpfile: &'a mut File,
}

pub fn command(rmodule: Rmodule) {

    if rmodule.command == "load" {
        load(rmodule.module, rmodule.shell);
    } else if rmodule.command == "unload" {
        unload(rmodule.module, rmodule.shell);
    } else if rmodule.command == "available" {
        available(rmodule.module, rmodule.modules, &rmodule.tmpfile);
    } else if rmodule.command == "list" {
        list(rmodule.module, rmodule.shell);
    } else if rmodule.command == "purge" {
        purge(rmodule.module, rmodule.shell);
    }
}

fn execute_command_and_parse() {
    // we basicly execute a system call (sourcing two bash files, one of them is the module)
    // then we parse the output of the call (the output of env)
    // and replace all the variables BLAH=124
    // with export|setenv BLAH(=)124
}

fn parse_modulefile() -> bool {
    return true;
    // execute_command_and_parse(strva(". /home/frlae/svn/frlae/modules/module_load_tools.sh && . %s && env", path), &buffer, shell);
}

fn load(module: &str, shell: &str) {
    println_stderr!("echo 'load {} {}'", module, shell);

    // check if module file exists

    // if not, maybe check if its a partial match
    // blast -> blast/x86_64/1.0 and blast/x86_64/2.0
    // then we need to load the Default version
    // or just the latest one

    // check if we are already loaded (LOADEDMODULES env var)

    // we already know the path to the module file (see above)
    // parse the module file and if successful
    // add it to the LOADEDMODULES env var
    // else unload the module

}

fn unload(module: &str, shell: &str) {
    println_stderr!("echo 'unload {} {}'", module, shell);
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
    println_stderr!("echo 'list {} {}'", module, shell);
}

fn purge(module: &str, shell: &str) {
    println_stderr!("echo 'purge {} {}'", module, shell);
}
