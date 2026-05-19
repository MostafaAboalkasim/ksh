use std::io::{self, Write};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn main() {
    let mut shell_commands = HashSet::new();
    shell_commands.insert("echo");
    shell_commands.insert("exit");
    shell_commands.insert("type");

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        if command.trim() == "exit" {
            break;
        } else if command.split_whitespace().next().unwrap_or("") == "echo" {
            println!("{}", command.split_whitespace().skip(1).collect::<Vec<_>>().join(" "));
        } else if command.split_whitespace().next().unwrap_or("") == "type" {
            if let Some(cmd) = command.split_whitespace().nth(1) {
                if shell_commands.contains(cmd) {
                    println!("{} is a shell builtin", cmd);
                    continue;
                } 
                let path_var = env::var("PATH").unwrap_or_default();
                let mut found = false;
                for dir in env::split_paths(&path_var) {

                    let full_path = dir.join(cmd);

                    if let Ok(metadata) = fs::metadata(&full_path) {
                        if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
                        {
                            println!("{} is {}", cmd, full_path.display());
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    println!("{}: not found", cmd);
                }
            }
        }
        else {
            println!("{}: command not found", command.trim());
        }
    }   
}