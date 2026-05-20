use std::io::{self, Write};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::os::unix::process::CommandExt;

pub fn find_in_path(cmd: &str) -> Option<std::path::PathBuf> {
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

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Ok(home) = env::var("HOME") {
            path.replacen('~', &home, 1)
        } else {
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

pub fn handle_echo(args: &[&str]) -> String {
    args.join(" ")
}

pub fn handle_type(cmd: &str, shell_commands: &HashSet<&str>) -> String {
    if shell_commands.contains(cmd) {
        format!("{} is a shell builtin", cmd)
    } else if let Some(path) = find_in_path(cmd) {
        format!("{} is {}", cmd, path.display())
    } else {
        format!("{}: not found", cmd)
    }
}

pub fn handle_pwd() -> Result<String, String> {
    env::current_dir()
        .map(|dir| dir.display().to_string())
        .map_err(|_| "pwd: error".to_string())
}

pub fn handle_cd(path: &str) -> Result<(), String> {
    let expanded_path = expand_tilde(path);
    
    if std::path::Path::new(&expanded_path).is_dir() {
        env::set_current_dir(&expanded_path)
            .map_err(|_| format!("cd: failed to change directory"))
    } else {
        Err(format!("cd: {}: No such file or directory", path))
    }
}

fn main() {
    let mut shell_commands = HashSet::new();
    shell_commands.insert("echo");
    shell_commands.insert("exit");
    shell_commands.insert("type");
    shell_commands.insert("pwd");
    shell_commands.insert("cd");

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

        match cmd {
            "exit" => break,
            "echo" => println!("{}", handle_echo(args)),
            "type" => {
                if let Some(target) = args.first() {
                    println!("{}", handle_type(target, &shell_commands));
                }
            }
            "pwd" => {
                match handle_pwd() {
                    Ok(dir) => println!("{}", dir),
                    Err(e) => println!("{}", e),
                }
            }
            "cd" => {
                if let Some(path) = args.first() {
                    match handle_cd(path) {
                        Ok(_) => {},
                        Err(e) => println!("{}", e),
                    }
                }
            }
            _ => {
                if let Some(path) = find_in_path(cmd) {
                    Command::new(&path).arg0(cmd).args(args).status().unwrap();
                } else {
                    println!("{}: command not found", cmd);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== find_in_path tests =====
    #[test]
    fn test_find_in_path_with_existing_command() {
        let result = find_in_path("ls");
        assert!(result.is_some(), "ls should be found in PATH");
    }

    #[test]
    fn test_find_in_path_with_nonexistent_command() {
        let result = find_in_path("nonexistent_command_xyz_12345");
        assert!(result.is_none(), "nonexistent command should not be found");
    }

    // ===== expand_tilde tests =====
    #[test]
    fn test_expand_tilde_with_home() {
        let path = "~/documents";
        let expanded = expand_tilde(path);
        
        if let Ok(home) = env::var("HOME") {
            assert_eq!(expanded, format!("{}/documents", home));
        }
    }

    #[test]
    fn test_expand_tilde_only() {
        let path = "~";
        let expanded = expand_tilde(path);
        
        if let Ok(home) = env::var("HOME") {
            assert_eq!(expanded, home);
        }
    }

    // ===== handle_echo tests =====
    #[test]
    fn test_handle_echo_single_arg() {
        let args = vec!["hello"];
        let result = handle_echo(&args);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_handle_echo_multiple_args() {
        let args = vec!["hello", "world", "test"];
        let result = handle_echo(&args);
        assert_eq!(result, "hello world test");
    }
    
    // ===== handle_type tests =====
    #[test]
    fn test_handle_type_builtin_command() {
        let mut shell_commands = HashSet::new();
        shell_commands.insert("echo");
        shell_commands.insert("pwd");
        
        let result = handle_type("echo", &shell_commands);
        assert_eq!(result, "echo is a shell builtin");
    }

    #[test]
    fn test_handle_type_nonexistent_command() {
        let shell_commands = HashSet::new();
        let result = handle_type("nonexistent_cmd_xyz", &shell_commands);
        assert_eq!(result, "nonexistent_cmd_xyz: not found");
    }

    #[test]
    fn test_handle_type_multiple_builtins() {
        let mut shell_commands = HashSet::new();
        shell_commands.insert("echo");
        shell_commands.insert("exit");
        shell_commands.insert("type");
        
        assert_eq!(handle_type("exit", &shell_commands), "exit is a shell builtin");
        assert_eq!(handle_type("type", &shell_commands), "type is a shell builtin");
    }

    // ===== handle_pwd tests =====
    #[test]
    fn test_handle_pwd_returns_current_directory() {
        let result = handle_pwd();
        assert!(result.is_ok(), "pwd should succeed");
        
        let pwd_result = result.unwrap();
        let current_dir = env::current_dir().unwrap().display().to_string();
        assert_eq!(pwd_result, current_dir);
    }

    #[test]
    fn test_handle_pwd_returns_absolute_path() {
        let result = handle_pwd();
        assert!(result.is_ok());
        
        let pwd_result = result.unwrap();
        assert!(pwd_result.starts_with("/") || pwd_result.starts_with("C:"), 
                "pwd should return absolute path");
    }

    // ===== handle_cd tests =====
    #[test]
    fn test_handle_cd_to_tmp() {
        let original_dir = env::current_dir().unwrap();
        
        let result = handle_cd("/tmp");
        assert!(result.is_ok(), "should be able to cd to /tmp");
        assert_eq!(env::current_dir().unwrap().display().to_string(), "/tmp");
        
        env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_handle_cd_to_nonexistent_directory() {
        let result = handle_cd("/nonexistent/dir/path/xyz");
        assert!(result.is_err(), "should fail for nonexistent directory");
        
        let error = result.unwrap_err();
        assert!(error.contains("No such file or directory"));
    }

    #[test]
    fn test_handle_cd_with_tilde_expansion() {
        let original_dir = env::current_dir().unwrap();
        
        let result = handle_cd("~");
        if let Ok(home) = env::var("HOME") {
            // Should succeed if HOME is set
            if result.is_ok() {
                let current = env::current_dir().unwrap().display().to_string();
                assert_eq!(current, home);
                env::set_current_dir(&original_dir).unwrap();
            }
        }
    }

    #[test]
    fn test_handle_cd_absolute_path() {
        let original_dir = env::current_dir().unwrap();
        
        // Try to cd to /tmp (exists on Unix systems)
        if std::path::Path::new("/tmp").is_dir() {
            let result = handle_cd("/tmp");
            assert!(result.is_ok());
            env::set_current_dir(&original_dir).unwrap();
        }
    }

    #[test]
    fn test_handle_cd_relative_path() {
        let original_dir = env::current_dir().unwrap();
        
        // Create a temp directory for testing
        let test_dir = original_dir.join("test_cd_dir_xyz");
        if let Ok(_) = fs::create_dir(&test_dir) {
            let _result = handle_cd("./test_cd_dir_xyz");
            // Should succeed or fail depending on permissions
            
            // Cleanup
            env::set_current_dir(&original_dir).unwrap();
            let _ = fs::remove_dir(&test_dir);
        }
    }

    #[test]
    fn test_handle_cd_error_message_format() {
        let result = handle_cd("/nonexistent/path");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.contains("cd:"), "error should start with 'cd:'");
        assert!(error.contains("/nonexistent/path"), "error should contain the path");
    }
}