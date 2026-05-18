#[allow(unused_imports)]
use std::io::{self, Write};
use std::collections::HashSet;

fn main() {
    // refactor this later
    let mut shell_commands = HashSet::new();
    shell_commands.insert("echo");
    shell_commands.insert("cd");
    shell_commands.insert("ls");
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
        } else if command.split_whitespace().next().unwrap_or("") == "cd" {
            // Handle cd command
        } else if command.split_whitespace().next().unwrap_or("") == "ls" {
            // Handle ls command
        } else if command.split_whitespace().next().unwrap_or("") == "type" {
            if let Some(cmd) = command.split_whitespace().nth(1) {
                if shell_commands.contains(cmd) {
                    println!("{} is a shell builtin", cmd);
                } else {
                    println!("{}: not found", cmd);
                }
            }
        }
        else {
            println!("{}: command not found", command.trim());
        }
    }   
}