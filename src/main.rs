use std::io::{self, Write};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::os::unix::process::CommandExt;

fn find_in_path(cmd: &str) -> Option<std::path::PathBuf> {
    let path_var = env::var("PATH").unwrap_or_default();
    for dir in env::split_paths(&path_var) {
        let full_path = dir.join(cmd);
        if let Ok(metadata) = fs::metadata(&full_path) {
            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                return Some(full_path);
            }
        }
    }
    None
}

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

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];
        let args = &parts[1..];

        if cmd == "exit" {
            break;
        } else if cmd == "echo" {
            println!("{}", args.join(" "));
        } else if cmd == "type" {
            if let Some(target) = args.first() {
                if shell_commands.contains(target) {
                    println!("{} is a shell builtin", target);
                } else if let Some(path) = find_in_path(target) {
                    println!("{} is {}", target, path.display());
                } else {
                    println!("{}: not found", target);
                }
            }
        } else if let Some(path) = find_in_path(cmd) {
            // Found an external executable — run it
            Command::new(&path) .arg0(cmd) .args(args).status().unwrap();
        } else {
            println!("{}: command not found", cmd);
        }
    }
}