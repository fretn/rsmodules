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

fn print_usage(shell_error: bool) {

    println!("Usage: rmodules <shell> <load|unload|list|purge|available> [module name]"); 

    if shell_error == true {
        println!("Only tcsh and bash are supported");
    }
}

fn main() {

	let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage(false);
        return;
    }

    let shell: String = args.get(1).unwrap().clone();

    if !is_shell_supported(&shell) {
        print_usage(true);
        return; 
    }

    let command: String;
    let modulename: String;

    if args.len() >= 3 {
        command = args.get(2).unwrap().clone();

        if command == "load" || command == "unload" || command == "available" {
            if args.len() > 3 {
                modulename = args.get(3).unwrap().clone(); 
                //run_command(command, modulename);
            } else {
                print_usage(false);
                return; 
            }
        } else if command == "list" || command == "purge" {
            //run_command(command); 
        } else {
            print_usage(false);
            return; 
        }
    }

    for arg in args {
        println!("{}", arg);
    }
}
