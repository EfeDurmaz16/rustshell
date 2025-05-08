# RustShell

A cross-platform shell utility written in Rust that provides unified commands with natural language names that work across different operating systems (Windows, Linux, macOS).

## Purpose

Many developers work across multiple operating systems and may not be familiar with commands on all platforms. RustShell solves this by providing a consistent, intuitive command interface that automatically translates to the appropriate native commands for your current OS.

Key benefits:
- Use consistent, intuitive commands regardless of OS
- Learn one set of commands that work everywhere
- Natural language command names that are easy to remember
- Helpful for developers transitioning between Windows, Linux, and macOS

## Available Commands

| Natural Command | Traditional Equivalent | Description | Usage |
|----------------|------------------------|-------------|-------|
| `make_dir [-p] <dir>` | `mkdir` | Create a directory | `make_dir test` or `make_dir -p path/to/dir` |
| `create_file <file>` | `touch` | Create a file | `create_file newfile.txt` |
| `copy <src> <dst>` | `cp` | Copy a file | `copy source.txt dest.txt` |
| `move <src> <dst>` | `mv` | Move a file or directory | `move oldfile.txt newfile.txt` |
| `delete_file <file>` | `rm` | Delete a file | `delete_file file.txt` |
| `delete_dir [-r] <dir>` | `rmdir`/`rm -r` | Delete a directory | `delete_dir test` or `delete_dir -r test` |
| `change_dir <dir>` | `cd` | Change directory | `change_dir path/to/dir` |
| `list [dir]` | `ls`/`dir` | List directory contents | `list` or `list path/to/dir` |
| `where_am_i` | `pwd` | Print current working directory | `where_am_i` |
| `run <cmd> [args...]` | `exec` | Run a system command | `run echo Hello World` |
| `show <file>` | `cat` | Display file contents | `show myfile.txt` |
| `find <pattern> [dir]` | `find`/`grep` | Find files by name | `find .txt` or `find .txt /path/to/dir` |
| `compress <src> <dst>` | `zip`/`tar` | Create a zip archive | `compress myfiles output.zip` |
| `help` | `help` | Show command help | `help` |

Note: The traditional shell commands (mkdir, ls, etc.) also work with this tool.

## OS-Specific Behaviors

While most commands use Rust's native cross-platform libraries, some commands have OS-specific implementations:

- `list`: Uses native formatting for each OS (Windows `dir` vs Unix `ls -la`)
- `run`: Runs commands through the appropriate shell (`cmd` on Windows, default shell on Unix)
- `find`: Uses PowerShell's Get-ChildItem on Windows and find on Unix
- `compress`: Uses PowerShell's Compress-Archive on Windows and zip on Unix

## Installation

1. Clone this repository
2. Build the project: `cargo build --release`
3. The executable will be available at `target/release/rustshell`

## Usage

Run the shell with commands directly:

```
rustshell make_dir test
rustshell create_file file.txt
rustshell list
rustshell where_am_i
rustshell run echo Hello World
rustshell show config.txt
rustshell find .log
rustshell compress documents archive.zip
```

## Future Enhancements

- Interactive shell mode
- More advanced commands
- Tab completion
- Piping between commands
- Support for aliases and custom commands 