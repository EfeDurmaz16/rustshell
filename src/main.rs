use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};


#[allow(dead_code)]
#[allow(unused_imports)]
// Define the Command trait which will implement platform-specific commands
trait ShellCommand {
    fn execute(&self) -> io::Result<()>;
    fn help(&self) -> String;
}

// OS-specific command implementations
mod commands {
    use super::*;

    pub struct MakeDir {
        pub path: String,
        pub parents: bool,
    }

    impl ShellCommand for MakeDir {
        fn execute(&self) -> io::Result<()> {
            // Use Rust's native functions for cross-platform compatibility
            let path = Path::new(&self.path);
            
            if self.parents {
                fs::create_dir_all(path)
            } else {
                fs::create_dir(path)
            }
        }

        fn help(&self) -> String {
            "Create a directory. Usage: mkdir [-p] <directory_name>".to_string()
        }
    }

    pub struct MakeFile {
        pub path: String,
    }

    impl ShellCommand for MakeFile {
        fn execute(&self) -> io::Result<()> {
            // Cross-platform file creation using Rust
            File::create(&self.path)?;
            Ok(())
        }

        fn help(&self) -> String {
            "Create a new empty file. Usage: touch <file_name>".to_string()
        }
    }

    pub struct CopyFile {
        pub src: String,
        pub dst: String,
    }

    impl ShellCommand for CopyFile {
        fn execute(&self) -> io::Result<()> {
            // Use native Rust for basic file operations
            fs::copy(&self.src, &self.dst)?;
            Ok(())
        }

        fn help(&self) -> String {
            "Copy a file. Usage: copy <source> <destination>".to_string()
        }
    }

    pub struct MoveFile {
        pub src: String,
        pub dst: String,
    }

    impl ShellCommand for MoveFile {
        fn execute(&self) -> io::Result<()> {
            fs::rename(&self.src, &self.dst)?;
            Ok(())
        }

        fn help(&self) -> String {
            "Move a file or directory. Usage: move <source> <destination>".to_string()
        }
    }

    pub struct RemoveFile {
        pub path: String,
    }

    impl ShellCommand for RemoveFile {
        fn execute(&self) -> io::Result<()> {
            fs::remove_file(&self.path)?;
            Ok(())
        }

        fn help(&self) -> String {
            "Remove a file. Usage: rm <file_name>".to_string()
        }
    }

    pub struct RemoveDir {
        pub path: String,
        pub recursive: bool,
    }

    impl ShellCommand for RemoveDir {
        fn execute(&self) -> io::Result<()> {
            if self.recursive {
                fs::remove_dir_all(&self.path)?;
            } else {
                fs::remove_dir(&self.path)?;
            }
            Ok(())
        }

        fn help(&self) -> String {
            "Remove a directory. Usage: rmdir [-r] <directory_name>".to_string()
        }
    }

    pub struct ChangeDir {
        pub path: String,
    }

    impl ShellCommand for ChangeDir {
        fn execute(&self) -> io::Result<()> {
            env::set_current_dir(&self.path)?;
            Ok(())
        }

        fn help(&self) -> String {
            "Change current directory. Usage: cd <directory_path>".to_string()
        }
    }

    pub struct ListDir {
        pub path: Option<String>,
    }

    impl ShellCommand for ListDir {
        fn execute(&self) -> io::Result<()> {
            let path = match &self.path {
                Some(p) => p,
                None => ".",
            };

            // Use either native Rust or OS-specific commands based on complexity
            if cfg!(windows) {
                // On Windows, use dir command with formatting
                // Use PowerShell to get better formatting and current directory resolution
                let output = Command::new("powershell")
                    .args(&["-Command", &format!("Get-ChildItem -Path \"{}\" | Format-Table -Property Mode, Name", path)])
                    .output()?;
                
                println!("Contents of {}:", path);
                print_output(output);
            } else {
                // On Unix-like systems, use ls command
                let ls_arg = if cfg!(target_os = "macos") {
                    "-la"
                } else {
                    "-la --color=auto"
                };
                
                let output = Command::new("sh")
                    .args(&["-c", &format!("ls {} \"{}\"", ls_arg, path)])
                    .output()?;
                
                println!("Contents of {}:", path);
                print_output(output);
            }
            
            Ok(())
        }

        fn help(&self) -> String {
            "List directory contents. Usage: ls [directory_path]".to_string()
        }
    }

    // Execute OS command with arguments
    pub struct ExecuteCommand {
        pub command: String,
        pub args: Vec<String>,
    }

    impl ShellCommand for ExecuteCommand {
        fn execute(&self) -> io::Result<()> {
            let output = if cfg!(windows) {
                let cmd_args = format!("{} {}", self.command, self.args.join(" "));
                Command::new("cmd")
                    .args(&["/C", &cmd_args])
                    .output()?
            } else {
                Command::new(&self.command)
                    .args(&self.args)
                    .output()?
            };
            
            print_output(output);
            Ok(())
        }

        fn help(&self) -> String {
            format!("Execute command: {} {}", self.command, self.args.join(" "))
        }
    }

    pub struct CurrentPath {}

    impl ShellCommand for CurrentPath {
        fn execute(&self) -> io::Result<()> {
            let current_dir = env::current_dir()?;
            println!("Current directory: {}", current_dir.display());
            Ok(())
        }

        fn help(&self) -> String {
            "Print current working directory. Usage: pwd".to_string()
        }
    }

    // New command to show file contents
    pub struct ShowFile {
        pub path: String,
    }

    impl ShellCommand for ShowFile {
        fn execute(&self) -> io::Result<()> {
            // Handle the file in a more robust way that works with non-UTF-8 content
            if cfg!(windows) {
                // On Windows, use PowerShell to display file content
                let output = Command::new("powershell")
                    .args(&["-Command", &format!("Get-Content -Path \"{}\"", self.path)])
                    .output()?;
                
                println!("--- Contents of {} ---", self.path);
                print_output(output);
            } else {
                // On Unix systems, use cat
                let output = Command::new("sh")
                    .args(&["-c", &format!("cat \"{}\"", self.path)])
                    .output()?;
                
                println!("--- Contents of {} ---", self.path);
                print_output(output);
            }
            
            println!("--- End of file ---");
            Ok(())
        }

        fn help(&self) -> String {
            "Display the contents of a file. Usage: show <file_path>".to_string()
        }
    }

    // New command to find files
    pub struct FindFiles {
        pub pattern: String,
        pub path: Option<String>,
    }

    impl ShellCommand for FindFiles {
        fn execute(&self) -> io::Result<()> {
            let root = match &self.path {
                Some(p) => PathBuf::from(p),
                None => env::current_dir()?,
            };
            
            println!("Searching for files matching '{}' in {}...", 
                     self.pattern, root.display());
            
            // Use native command for better performance and features
            if cfg!(windows) {
                // Windows - use PowerShell
                let cmd = format!(
                    "Get-ChildItem -Path \"{}\" -Recurse -File | Where-Object {{ $_.Name -like \"*{}*\" }} | Select-Object FullName",
                    root.display(),
                    self.pattern
                );
                
                let output = Command::new("powershell")
                    .args(&["-Command", &cmd])
                    .output()?;
                
                print_output(output);
            } else {
                // Unix - use find
                let cmd = format!(
                    "find \"{}\" -type f -name \"*{}*\"",
                    root.display(),
                    self.pattern
                );
                
                let output = Command::new("sh")
                    .args(&["-c", &cmd])
                    .output()?;
                
                print_output(output);
            }
            
            Ok(())
        }

        fn help(&self) -> String {
            "Find files matching a pattern. Usage: find <pattern> [directory]".to_string()
        }
    }

    // New command to compress files into a zip archive
    pub struct CompressFiles {
        pub source: String,
        pub destination: String,
    }

    impl ShellCommand for CompressFiles {
        fn execute(&self) -> io::Result<()> {
            println!("Compressing {} to {}...", self.source, self.destination);
            
            if cfg!(windows) {
                // Windows compression using PowerShell
                let cmd = format!(
                    "Compress-Archive -Path \"{}\" -DestinationPath \"{}\" -Force",
                    self.source,
                    self.destination
                );
                
                let output = Command::new("powershell")
                    .args(&["-Command", &cmd])
                    .output()?;
                
                print_output(output);
            } else {
                // Unix compression using zip
                let output = Command::new("sh")
                    .args(&["-c", &format!("zip -r \"{}\" \"{}\"", 
                                         self.destination, self.source)])
                    .output()?;
                
                print_output(output);
            }
            
            println!("Compression complete.");
            Ok(())
        }

        fn help(&self) -> String {
            "Compress files into a zip archive. Usage: compress <source> <destination>".to_string()
        }
    }

    // Helper function to print command output
    fn print_output(output: Output) {
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }
}

fn parse_command(args: &[String]) -> Option<Box<dyn ShellCommand>> {
    if args.is_empty() {
        return None;
    }

    match args[0].as_str() {
        "make_dir" | "mkdir" => {
            if args.len() < 2 {
                println!("Error: make_dir requires a directory name");
                return None;
            }
            
            let mut parents = false;
            let mut path_index = 1;
            
            if args.len() > 2 && args[1] == "-p" {
                parents = true;
                path_index = 2;
            }
            
            Some(Box::new(commands::MakeDir {
                path: args[path_index].clone(),
                parents,
            }))
        },
        "create_file" | "touch" => {
            if args.len() < 2 {
                println!("Error: create_file requires a file name");
                return None;
            }
            
            Some(Box::new(commands::MakeFile {
                path: args[1].clone(),
            }))
        },
        "copy" => {
            if args.len() < 3 {
                println!("Error: copy requires source and destination paths");
                return None;
            }
            
            Some(Box::new(commands::CopyFile {
                src: args[1].clone(),
                dst: args[2].clone(),
            }))
        },
        "move" => {
            if args.len() < 3 {
                println!("Error: move requires source and destination paths");
                return None;
            }
            
            Some(Box::new(commands::MoveFile {
                src: args[1].clone(),
                dst: args[2].clone(),
            }))
        },
        "delete_file" | "rm" => {
            if args.len() < 2 {
                println!("Error: delete_file requires a file name");
                return None;
            }
            
            Some(Box::new(commands::RemoveFile {
                path: args[1].clone(),
            }))
        },
        "delete_dir" | "rmdir" => {
            if args.len() < 2 {
                println!("Error: delete_dir requires a directory name");
                return None;
            }
            
            let mut recursive = false;
            let mut path_index = 1;
            
            if args.len() > 2 && args[1] == "-r" {
                recursive = true;
                path_index = 2;
            }
            
            Some(Box::new(commands::RemoveDir {
                path: args[path_index].clone(),
                recursive,
            }))
        },
        "change_dir" | "cd" => {
            if args.len() < 2 {
                println!("Error: change_dir requires a directory path");
                return None;
            }
            
            Some(Box::new(commands::ChangeDir {
                path: args[1].clone(),
            }))
        },
        "list" | "ls" => {
            let path = if args.len() > 1 {
                Some(args[1].clone())
            } else {
                None
            };
            
            Some(Box::new(commands::ListDir { path }))
        },
        "where_am_i" | "pwd" => {
            Some(Box::new(commands::CurrentPath {}))
        },
        "run" | "exec" => {
            if args.len() < 2 {
                println!("Error: run requires a command to execute");
                return None;
            }
            
            let command = args[1].clone();
            let command_args: Vec<String> = args[2..].to_vec();
            
            Some(Box::new(commands::ExecuteCommand {
                command,
                args: command_args,
            }))
        },
        "show" | "cat" => {
            if args.len() < 2 {
                println!("Error: show requires a file path");
                return None;
            }
            
            Some(Box::new(commands::ShowFile {
                path: args[1].clone(),
            }))
        },
        "find" => {
            if args.len() < 2 {
                println!("Error: find requires a pattern to search for");
                return None;
            }
            
            let pattern = args[1].clone();
            let path = if args.len() > 2 {
                Some(args[2].clone())
            } else {
                None
            };
            
            Some(Box::new(commands::FindFiles {
                pattern,
                path,
            }))
        },
        "compress" | "zip" => {
            if args.len() < 3 {
                println!("Error: compress requires source and destination paths");
                return None;
            }
            
            Some(Box::new(commands::CompressFiles {
                source: args[1].clone(),
                destination: args[2].clone(),
            }))
        },
        "help" => {
            print_help();
            None
        },
        _ => {
            println!("Unknown command: {}", args[0]);
            println!("Use 'help' to see available commands");
            None
        }
    }
}

fn print_help() {
    let header = "Cross-Platform Shell - Available Commands:";
    let commands = [
        "  make_dir [-p] <directory>    Create a directory",
        "  create_file <file>           Create a file",
        "  copy <src> <dst>             Copy a file",
        "  move <src> <dst>             Move a file or directory",
        "  delete_file <file>           Delete a file",
        "  delete_dir [-r] <directory>  Delete a directory",
        "  change_dir <directory>       Change current directory",
        "  list [directory]             List directory contents",
        "  where_am_i                   Show current directory",
        "  run <cmd> [args...]          Run a system command",
        "  show <file>                  Display contents of a file",
        "  find <pattern> [dir]         Find files matching a pattern",
        "  compress <src> <dst>         Create a zip archive",
        "  help                         Show this help message",
    ];
    
    // Print each part separately to ensure everything is displayed
    println!("{}", header);
    for cmd in commands {
        println!("{}", cmd);
    }
    
    println!();
    println!("Note: Traditional shell commands (mkdir, ls, etc.) also work.");
    println!();
    
    // Print OS-specific information
    let os_info = if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown OS"
    };
    
    println!("Current OS: {}", os_info);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    
    if args.is_empty() {
        println!("Cross-Platform Shell");
        println!("Use 'help' to see available commands");
        return;
    }
    
    if let Some(command) = parse_command(&args) {
        if let Err(e) = command.execute() {
            eprintln!("Error executing command: {}", e);
        }
    }
}
