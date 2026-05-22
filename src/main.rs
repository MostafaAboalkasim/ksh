use std::io::{self, Write};
use std::env;
use std::fs; 
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::collections::VecDeque;
use std::path::PathBuf;

const HISTORY_LIMIT: usize = 1000;

struct Shell {
    history: VecDeque<String>,
}

impl Shell {
    fn new() -> Self {
        Self {
            history: VecDeque::new(),
        }
    }

    fn run(&mut self) {
        loop {
            print!("$ ");
            io::stdout().flush().unwrap();

            let Some(input) = Self::read_input() else {
                continue;
            };

            let trimmed = input.trim();

            if trimmed.is_empty() {
                continue;
            }

            self.history.push_back(trimmed.to_string());
            if self.history.len() > HISTORY_LIMIT {
                self.history.pop_front();
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let (cmd, args) = parts.split_first().unwrap();

            if self.handle_builtin(cmd, args) {
                continue;
            }

            self.execute_command(cmd, args);
        }
    }

    fn read_input() -> Option<String> {
        let mut input = String::new();

        io::stdin().read_line(&mut input).ok()?;

        Some(input)
    }

    fn handle_builtin(&mut self, cmd: &str, args: &[&str]) -> bool {
        match cmd {
            "exit" => std::process::exit(0),

            "echo" => {
                println!("{}", args.join(" "));
            }

            "pwd" => {
                match env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(err) => eprintln!("pwd: {err}"),
                }
            }

            "cd" => {
                let path = args.first().copied().unwrap_or("~");

                if let Err(err) = change_directory(path) {
                    eprintln!("cd: {err}");
                }
            }

            "history" => {
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("{} {}", i + 1, cmd);
                }
            }

            "env" => {
                for (key, value) in env::vars() {
                    println!("{key}={value}");
                }
            }

            "export" => {
                if let Some(var) = args.first() {
                    export_variable(var);
                }
            }

            "unset" => {
                if let Some(key) = args.first() {
                    unsafe {
                        env::remove_var(key);
                    }
                }
            }

            _ => return false,
        }

        true
    }

    fn execute_command(&self, cmd: &str, args: &[&str]) {
        let Some(path) = find_executable(cmd) else {
            eprintln!("{cmd}: command not found");
            return;
        };

        if let Err(err) = Command::new(&path).arg0(cmd).args(args).status() {
            eprintln!("{cmd}: {err}");
        }
    }
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;

    env::split_paths(&path_var).find_map(|dir| {
        let full_path = dir.join(cmd);

        let metadata = fs::metadata(&full_path).ok()?;

        let is_executable =
            metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0);

        is_executable.then_some(full_path)
    })
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" || path.starts_with("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(path.replacen('~', &home.to_string_lossy(), 1));
        }
    }

    PathBuf::from(path)
}

fn change_directory(path: &str) -> io::Result<()> {
    let expanded = expand_tilde(path);

    if !expanded.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}: No such directory", expanded.display()),
        ));
    }

    env::set_current_dir(expanded)
}

fn export_variable(arg: &str) {
    if let Some((key, value)) = arg.split_once('=') {
        unsafe {
            env::set_var(key, value);
        }
    }
}

fn main() {
    Shell::new().run();
}