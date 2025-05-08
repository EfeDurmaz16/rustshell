use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::collections::HashMap;


#[allow(dead_code)]
#[allow(unused_imports)]
// Define the Command trait which will implement platform-specific commands
trait ShellCommand {
    fn execute(&self) -> io::Result<()>;
    fn help(&self) -> String;
}

// Alias manager
struct AliasManager {
    aliases: HashMap<String, String>,
    alias_file: PathBuf,
}

impl AliasManager {
    fn new() -> io::Result<Self> {
        let home_dir = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let alias_file = home_dir.join(".rustshell_aliases");
        
        let mut alias_manager = AliasManager {
            aliases: HashMap::new(),
            alias_file,
        };
        
        // Load aliases from file if it exists
        alias_manager.load_aliases()?;
        
        Ok(alias_manager)
    }
    
    fn load_aliases(&mut self) -> io::Result<()> {
        if !self.alias_file.exists() {
            return Ok(());
        }
        
        let file = File::open(&self.alias_file)?;
        let reader = std::io::BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            if let Some(pos) = line.find('=') {
                let name = line[..pos].trim().to_string();
                let command = line[pos+1..].trim().to_string();
                self.aliases.insert(name, command);
            }
        }
        
        Ok(())
    }
    
    fn save_aliases(&self) -> io::Result<()> {
        let mut file = File::create(&self.alias_file)?;
        
        writeln!(file, "# RustShell aliases")?;
        for (name, command) in &self.aliases {
            writeln!(file, "{}={}", name, command)?;
        }
        
        Ok(())
    }
    
    fn add_alias(&mut self, name: String, command: String) -> io::Result<()> {
        self.aliases.insert(name, command);
        self.save_aliases()
    }
    
    fn remove_alias(&mut self, name: &str) -> io::Result<bool> {
        let existed = self.aliases.remove(name).is_some();
        if existed {
            self.save_aliases()?;
        }
        Ok(existed)
    }
    
    fn get_alias(&self, name: &str) -> Option<&String> {
        self.aliases.get(name)
    }
    
    fn list_aliases(&self) {
        if self.aliases.is_empty() {
            println!("No aliases defined.");
            return;
        }
        
        println!("Defined aliases:");
        for (name, command) in &self.aliases {
            println!("  {} = '{}'", name, command);
        }
    }
    
    fn expand_aliases(&self, args: &[String]) -> Vec<String> {
        if args.is_empty() {
            return Vec::new();
        }
        
        // Check if the command is an alias
        if let Some(alias_cmd) = self.get_alias(&args[0]) {
            // Split the alias command into words
            let mut expanded: Vec<String> = alias_cmd
                .split_whitespace()
                .map(String::from)
                .collect();
            
            // Add any additional arguments from the original command
            if args.len() > 1 {
                expanded.extend_from_slice(&args[1..]);
            }
            
            return expanded;
        }
        
        // Not an alias, return the original args
        args.to_vec()
    }
}

// Helper struct for rustyline tab completion and other functionality
#[derive(Helper, Highlighter, Hinter, Validator)]
struct RustShellHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    validator: MatchingBracketValidator,
    commands: Vec<String>,
    alias_manager: AliasManager,
}

impl RustShellHelper {
    fn new(alias_manager: AliasManager) -> Self {
        let mut commands = vec![
            "make_dir".to_string(), "mkdir".to_string(),
            "create_file".to_string(), "touch".to_string(),
            "copy".to_string(), 
            "move".to_string(),
            "delete_file".to_string(), "rm".to_string(),
            "delete_dir".to_string(), "rmdir".to_string(),
            "change_dir".to_string(), "cd".to_string(),
            "list".to_string(), "ls".to_string(),
            "where_am_i".to_string(), "pwd".to_string(),
            "run".to_string(), "exec".to_string(),
            "show".to_string(), "cat".to_string(),
            "find".to_string(),
            "compress".to_string(), "zip".to_string(),
            "help".to_string(),
            "exit".to_string(), "quit".to_string(),
            "interactive".to_string(),
            "alias".to_string(),
            "unalias".to_string(),
            "pipe".to_string(),
        ];
        
        // Add aliases to command completions
        for alias in alias_manager.aliases.keys() {
            commands.push(alias.clone());
        }
        
        RustShellHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
            commands,
            alias_manager,
        }
    }
    
    fn update_commands(&mut self) {
        // Update command list with current aliases
        let mut base_commands = vec![
            "make_dir".to_string(), "mkdir".to_string(),
            "create_file".to_string(), "touch".to_string(),
            "copy".to_string(), 
            "move".to_string(),
            "delete_file".to_string(), "rm".to_string(),
            "delete_dir".to_string(), "rmdir".to_string(),
            "change_dir".to_string(), "cd".to_string(),
            "list".to_string(), "ls".to_string(),
            "where_am_i".to_string(), "pwd".to_string(),
            "run".to_string(), "exec".to_string(),
            "show".to_string(), "cat".to_string(),
            "find".to_string(),
            "compress".to_string(), "zip".to_string(),
            "help".to_string(),
            "exit".to_string(), "quit".to_string(),
            "interactive".to_string(),
            "alias".to_string(),
            "unalias".to_string(),
            "pipe".to_string(),
        ];
        
        // Add aliases
        for alias in self.alias_manager.aliases.keys() {
            base_commands.push(alias.clone());
        }
        
        self.commands = base_commands;
    }
}

impl Completer for RustShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // First try to complete the command
        if !line.contains(' ') || pos <= line.find(' ').unwrap_or(line.len()) {
            let mut command_matches = Vec::new();
            
            // Filter commands that match the current word
            for cmd in &self.commands {
                if cmd.starts_with(line) {
                    command_matches.push(Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    });
                }
            }
            
            if !command_matches.is_empty() {
                return Ok((0, command_matches));
            }
        }
        
        // If not a command or after the command, use filename completion
        self.completer.complete(line, pos, ctx)
    }
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
        pub paths: Vec<String>,
    }

    impl ShellCommand for MakeFile {
        fn execute(&self) -> io::Result<()> {
            // Create multiple files
            for path in &self.paths {
                println!("Creating file: {}", path);
                File::create(path)?;
            }
            Ok(())
        }

        fn help(&self) -> String {
            "Create one or more empty files. Usage: create_file <file1> <file2> ...".to_string()
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
        pub paths: Vec<String>,
    }

    impl ShellCommand for RemoveFile {
        fn execute(&self) -> io::Result<()> {
            // Remove multiple files
            for path in &self.paths {
                println!("Removing file: {}", path);
                fs::remove_file(path)?;
            }
            Ok(())
        }

        fn help(&self) -> String {
            "Remove one or more files. Usage: delete_file <file1> <file2> ...".to_string()
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

    // Add new commands for aliases
    pub struct AliasCommand {
        pub name: Option<String>,
        pub command: Option<String>,
    }

    impl super::ShellCommand for AliasCommand {
        fn execute(&self) -> std::io::Result<()> {
            let alias_manager = super::AliasManager::new()?;
            
            match (&self.name, &self.command) {
                (Some(name), Some(command)) => {
                    // Add or update alias
                    let mut manager = alias_manager;
                    manager.add_alias(name.clone(), command.clone())?;
                    println!("Alias '{}' created for '{}'", name, command);
                }
                (None, None) => {
                    // List all aliases
                    alias_manager.list_aliases();
                }
                _ => {
                    println!("Error: alias requires both name and command or no arguments");
                }
            }
            
            Ok(())
        }

        fn help(&self) -> String {
            "Create or list aliases. Usage: alias [name='command']".to_string()
        }
    }

    pub struct UnaliasCommand {
        pub name: String,
    }

    impl super::ShellCommand for UnaliasCommand {
        fn execute(&self) -> std::io::Result<()> {
            let mut alias_manager = super::AliasManager::new()?;
            
            match alias_manager.remove_alias(&self.name) {
                Ok(true) => println!("Alias '{}' removed", self.name),
                Ok(false) => println!("No such alias: {}", self.name),
                Err(e) => return Err(e),
            }
            
            Ok(())
        }

        fn help(&self) -> String {
            "Remove an alias. Usage: unalias <name>".to_string()
        }
    }
    
    // Command for pipeline execution
    pub struct PipeCommand {
        pub commands: Vec<Vec<String>>,
    }

    impl super::ShellCommand for PipeCommand {
        fn execute(&self) -> std::io::Result<()> {
            if self.commands.len() < 2 {
                println!("Error: pipe requires at least two commands");
                return Ok(());
            }
            
            // Setup for piping
            let mut previous_stdout = None;
            
            for (i, cmd_args) in self.commands.iter().enumerate() {
                if cmd_args.is_empty() {
                    println!("Error: empty command in pipeline");
                    return Ok(());
                }
                
                let is_last = i == self.commands.len() - 1;
                
                // Create the command
                let mut cmd = if cfg!(windows) {
                    let cmd_str = cmd_args.join(" ");
                    let mut command = std::process::Command::new("cmd");
                    command.args(&["/C", &cmd_str]);
                    command
                } else {
                    let mut command = std::process::Command::new(&cmd_args[0]);
                    if cmd_args.len() > 1 {
                        command.args(&cmd_args[1..]);
                    }
                    command
                };
                
                // Setup stdin from previous command's stdout if available
                if let Some(stdout) = previous_stdout {
                    cmd.stdin(stdout);
                }
                
                // Setup stdout for piping to next command or capturing output
                if !is_last {
                    cmd.stdout(Stdio::piped());
                }
                
                // Execute the command
                let mut child = cmd.spawn()?;
                
                // Get stdout for the next command in the pipeline
                previous_stdout = if !is_last {
                    child.stdout.take()
                } else {
                    None
                };
                
                // If it's the last command, wait for it to finish
                if is_last {
                    let status = child.wait()?;
                    if !status.success() {
                        println!("Pipeline command failed with exit code: {:?}", status.code());
                    }
                }
            }
            
            Ok(())
        }

        fn help(&self) -> String {
            "Execute commands in a pipeline. Usage: pipe 'cmd1' 'cmd2' ...".to_string()
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

fn parse_command(args: &[String], alias_manager: Option<&AliasManager>) -> Option<Box<dyn ShellCommand>> {
    if args.is_empty() {
        return None;
    }
    
    // Expand aliases if alias_manager is provided
    let expanded_args = if let Some(manager) = alias_manager {
        manager.expand_aliases(args)
    } else {
        args.to_vec()
    };
    
    if expanded_args.is_empty() {
        return None;
    }
    
    match expanded_args[0].as_str() {
        "make_dir" | "mkdir" => {
            if expanded_args.len() < 2 {
                println!("Error: make_dir requires a directory name");
                return None;
            }
            
            let mut parents = false;
            let mut path_index = 1;
            
            if expanded_args.len() > 2 && expanded_args[1] == "-p" {
                parents = true;
                path_index = 2;
            }
            
            Some(Box::new(commands::MakeDir {
                path: expanded_args[path_index].clone(),
                parents,
            }))
        },
        "create_file" | "touch" => {
            if expanded_args.len() < 2 {
                println!("Error: create_file requires at least one file name");
                return None;
            }
            
            // Skip the command name and collect all file paths
            let paths = expanded_args[1..].to_vec();
            
            Some(Box::new(commands::MakeFile { paths }))
        },
        "copy" => {
            if expanded_args.len() < 3 {
                println!("Error: copy requires source and destination paths");
                return None;
            }
            
            Some(Box::new(commands::CopyFile {
                src: expanded_args[1].clone(),
                dst: expanded_args[2].clone(),
            }))
        },
        "move" => {
            if expanded_args.len() < 3 {
                println!("Error: move requires source and destination paths");
                return None;
            }
            
            Some(Box::new(commands::MoveFile {
                src: expanded_args[1].clone(),
                dst: expanded_args[2].clone(),
            }))
        },
        "delete_file" | "rm" => {
            if expanded_args.len() < 2 {
                println!("Error: delete_file requires at least one file name");
                return None;
            }
            
            // Skip the command name and collect all file paths
            let paths = expanded_args[1..].to_vec();
            
            Some(Box::new(commands::RemoveFile { paths }))
        },
        "delete_dir" | "rmdir" => {
            if expanded_args.len() < 2 {
                println!("Error: delete_dir requires a directory name");
                return None;
            }
            
            let mut recursive = false;
            let mut path_index = 1;
            
            if expanded_args.len() > 2 && expanded_args[1] == "-r" {
                recursive = true;
                path_index = 2;
            }
            
            Some(Box::new(commands::RemoveDir {
                path: expanded_args[path_index].clone(),
                recursive,
            }))
        },
        "change_dir" | "cd" => {
            if expanded_args.len() < 2 {
                println!("Error: change_dir requires a directory path");
                return None;
            }
            
            Some(Box::new(commands::ChangeDir {
                path: expanded_args[1].clone(),
            }))
        },
        "list" | "ls" => {
            let path = if expanded_args.len() > 1 {
                Some(expanded_args[1].clone())
            } else {
                None
            };
            
            Some(Box::new(commands::ListDir { path }))
        },
        "where_am_i" | "pwd" => {
            Some(Box::new(commands::CurrentPath {}))
        },
        "run" | "exec" => {
            if expanded_args.len() < 2 {
                println!("Error: run requires a command to execute");
                return None;
            }
            
            let command = expanded_args[1].clone();
            let command_args: Vec<String> = expanded_args[2..].to_vec();
            
            Some(Box::new(commands::ExecuteCommand {
                command,
                args: command_args,
            }))
        },
        "show" | "cat" => {
            if expanded_args.len() < 2 {
                println!("Error: show requires a file path");
                return None;
            }
            
            Some(Box::new(commands::ShowFile {
                path: expanded_args[1].clone(),
            }))
        },
        "find" => {
            if expanded_args.len() < 2 {
                println!("Error: find requires a pattern to search for");
                return None;
            }
            
            let pattern = expanded_args[1].clone();
            let path = if expanded_args.len() > 2 {
                Some(expanded_args[2].clone())
            } else {
                None
            };
            
            Some(Box::new(commands::FindFiles {
                pattern,
                path,
            }))
        },
        "compress" | "zip" => {
            if expanded_args.len() < 3 {
                println!("Error: compress requires source and destination paths");
                return None;
            }
            
            Some(Box::new(commands::CompressFiles {
                source: expanded_args[1].clone(),
                destination: expanded_args[2].clone(),
            }))
        },
        "alias" => {
            if expanded_args.len() == 1 {
                // List aliases
                Some(Box::new(commands::AliasCommand {
                    name: None,
                    command: None,
                }))
            } else if expanded_args.len() >= 3 {
                // Create alias: alias name command args...
                let name = expanded_args[1].clone();
                let command = expanded_args[2..].join(" ");
                
                Some(Box::new(commands::AliasCommand {
                    name: Some(name),
                    command: Some(command),
                }))
            } else {
                println!("Error: invalid alias syntax. Use: alias <name> <command>");
                None
            }
        },
        "unalias" => {
            if expanded_args.len() < 2 {
                println!("Error: unalias requires an alias name");
                return None;
            }
            
            Some(Box::new(commands::UnaliasCommand {
                name: expanded_args[1].clone(),
            }))
        },
        "pipe" => {
            if expanded_args.len() < 3 {
                println!("Error: pipe requires at least two commands");
                return None;
            }
            
            // Parse pipe commands - each argument becomes a separate command in the pipeline
            let commands: Vec<Vec<String>> = expanded_args[1..].iter()
                .map(|cmd_str| {
                    cmd_str.split_whitespace()
                        .map(String::from)
                        .collect()
                })
                .collect();
            
            Some(Box::new(commands::PipeCommand { commands }))
        },
        "help" => {
            print_help();
            None
        },
        _ => {
            println!("Unknown command: {}", expanded_args[0]);
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
        "  alias [name command]         Create or list aliases",
        "  unalias <name>               Remove an alias",
        "  pipe 'cmd1' 'cmd2' ...       Connect commands with pipes",
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

// Function to run in interactive mode
fn run_interactive_mode() -> io::Result<()> {
    // Create config with rustyline 11.0.0 compatible settings
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    // Create editor and load alias manager
    let alias_manager = match AliasManager::new() {
        Ok(am) => am,
        Err(e) => {
            eprintln!("Error loading alias manager: {}", e);
            return Err(e);
        }
    };
    
    let helper = RustShellHelper::new(alias_manager);
    
    // Create editor with config
    let mut rl = match Editor::with_config(config) {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Error creating editor: {:?}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to create editor"));
        }
    };
    
    // Set helper for editor
    rl.set_helper(Some(helper));
    
    // Try to load history
    let history_path = Path::new(".rustshell_history");
    if rl.load_history(history_path).is_err() {
        println!("No previous history.");
    }
    
    // Print welcome message
    let os_info = if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown OS"
    };
    
    println!("RustShell Interactive Mode - {}", os_info);
    println!("Type 'help' for a list of commands or 'exit' to quit.");
    
    // Interactive loop
    loop {
        let current_dir = env::current_dir()?;
        let prompt = format!("{}> ", current_dir.display());
        
        match rl.readline(&prompt) {
            Ok(line) => {
                // Add to history if non-empty
                if !line.trim().is_empty() {
                    let _ = rl.add_history_entry(&line);
                }
                
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }
                
                // Handle exit commands
                if line == "exit" || line == "quit" {
                    println!("Goodbye!");
                    break;
                }
                
                // Parse the command line
                let args: Vec<String> = line
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                
                // Get alias manager from helper to handle aliases
                if let Some(helper) = rl.helper_mut() {
                    if let Some(command) = parse_command(&args, Some(&helper.alias_manager)) {
                        if let Err(e) = command.execute() {
                            eprintln!("Error executing command: {}", e);
                        }
                    }
                    
                    // Update commands to include any new aliases
                    helper.update_commands();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    
    // Save history
    if let Err(e) = rl.save_history(history_path) {
        eprintln!("Error saving command history: {}", e);
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check if we should run in interactive mode (no arguments or explicit "interactive" argument)
    if args.len() <= 1 || (args.len() == 2 && args[1] == "interactive") {
        if let Err(e) = run_interactive_mode() {
            eprintln!("Error in interactive mode: {}", e);
        }
        return;
    }
    
    // Otherwise, run in command mode
    let command_args: Vec<String> = args.iter().skip(1).cloned().collect();
    
    // We can't use aliases in non-interactive mode
    if let Some(command) = parse_command(&command_args, None) {
        if let Err(e) = command.execute() {
            eprintln!("Error executing command: {}", e);
        }
    }
}
