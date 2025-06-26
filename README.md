# RustShell

A cross-platform shell utility written in Rust that provides unified commands with natural language names that work across different operating systems (Windows, Linux, macOS).

## Purpose

Many developers work across multiple operating systems and may not be familiar with commands on all platforms. RustShell solves this by providing a consistent, intuitive command interface that automatically translates to the appropriate native commands for your current OS.

Key benefits:
- **Natural Language Processing**: Use AI to translate plain English to shell commands
- Use consistent, intuitive commands regardless of OS
- Learn one set of commands that work everywhere
- Natural language command names that are easy to remember
- Helpful for developers transitioning between Windows, Linux, and macOS
- Interactive shell mode with tab completion and command history
- Support for aliases and command pipelines
- Safety checks for destructive operations

## Available Commands

| Natural Command | Traditional Equivalent | Description | Usage |
|----------------|------------------------|-------------|-------|
| `make_dir [-p] <dir>` | `mkdir` | Create a directory | `make_dir test` or `make_dir -p path/to/dir` |
| `create_file <file1> [file2...]` | `touch` | Create one or more files | `create_file file1.txt file2.txt` |
| `copy <src> <dst>` | `cp` | Copy a file | `copy source.txt dest.txt` |
| `move <src> <dst>` | `mv` | Move a file or directory | `move oldfile.txt newfile.txt` |
| `delete_file <file1> [file2...]` | `rm` | Delete one or more files | `delete_file file1.txt file2.txt` |
| `delete_dir [-r] <dir>` | `rmdir`/`rm -r` | Delete a directory | `delete_dir test` or `delete_dir -r test` |
| `change_dir <dir>` | `cd` | Change directory | `change_dir path/to/dir` |
| `list [dir]` | `ls`/`dir` | List directory contents | `list` or `list path/to/dir` |
| `where_am_i` | `pwd` | Print current working directory | `where_am_i` |
| `run <cmd> [args...]` | `exec` | Run a system command | `run echo Hello World` |
| `show <file>` | `cat` | Display file contents | `show myfile.txt` |
| `find <pattern> [dir]` | `find`/`grep` | Find files by name | `find .txt` or `find .txt /path/to/dir` |
| `compress <src> <dst>` | `zip`/`tar` | Create a zip archive | `compress myfiles output.zip` |
| `alias [name command]` | `alias` | Create or list aliases | `alias ll list -la` |
| `unalias <name>` | `unalias` | Remove an alias | `unalias ll` |
| `pipe 'cmd1' 'cmd2'` | `|` | Connect commands with pipes | `pipe 'list' 'grep txt'` |
| `help` | `help` | Show command help | `help` |

Note: The traditional shell commands (mkdir, ls, etc.) also work with this tool.

## OS-Specific Behaviors

While most commands use Rust's native cross-platform libraries, some commands have OS-specific implementations:

- `list`: Uses native formatting for each OS (Windows `dir` vs Unix `ls -la`)
- `run`: Runs commands through the appropriate shell (`cmd` on Windows, default shell on Unix)
- `find`: Uses PowerShell's Get-ChildItem on Windows and find on Unix
- `compress`: Uses PowerShell's Compress-Archive on Windows and zip on Unix

## Installation

### Global Installation (Recommended)

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd rustshell
   ```

2. Install globally:
   ```bash
   cargo install --path .
   ```

3. Set up your API key (see Configuration section below)

4. Use from anywhere:
   ```bash
   rustshell "npm run dev on port 3001"
   rustshell "create a directory called test"
   rustshell "list all files"
   ```

### Local Development

1. Clone this repository
2. Build the project: `cargo build --release`
3. Run with: `cargo run -- "your command"`

## Configuration

RustShell supports natural language processing using OpenAI's API. To enable this feature:

### Option 1: .env File (Recommended)

1. **Set up global configuration**:
   ```bash
   mkdir -p ~/.rustshell
   cp .env.example ~/.rustshell/.env
   ```

2. **Add your API key**:
   Edit `~/.rustshell/.env`:
   ```bash
   # RustShell Environment Variables
   OPENAI_API_KEY=your-actual-openai-api-key-here
   ```

### Option 2: Environment Variable

Set the environment variable globally:
```bash
export OPENAI_API_KEY="your-api-key-here"
# Add to your ~/.bashrc or ~/.zshrc to make it permanent
```

### Option 3: Config File

1. **Copy the configuration template**:
   ```bash
   mkdir -p ~/.rustshell
   cp config/rustshell.toml ~/.rustshell/config.toml
   ```

2. **Edit the config file**:
   ```toml
   [llm]
   provider = "openai"
   model = "gpt-3.5-turbo"
   api_key_env = "OPENAI_API_KEY"  # Environment variable name
   ```

### .env File Priority

RustShell looks for `.env` files in this order:
1. Current directory (`./.env`)
2. RustShell config directory (`~/.rustshell/.env`)
3. Home directory (`~/.env`)

### Natural Language Examples

Once configured, you can use natural language commands:

```bash
rustshell "create a directory called my_project"
rustshell "show me what files are in this folder"  
rustshell "copy all text files to the backup folder"
rustshell "remove the temporary files"
```

## Usage

### Command Mode

Run the shell with commands directly:

```
rustshell make_dir test
rustshell create_file file.txt document.md
rustshell list
rustshell where_am_i
rustshell run echo Hello World
rustshell show config.txt
rustshell find .log
rustshell compress documents archive.zip
```

### Interactive Mode

Run the shell in interactive mode for a more traditional shell experience:

```
rustshell
```

or

```
rustshell interactive
```

Features in interactive mode:
- Tab completion for commands and file paths
- Command history (stored in `.rustshell_history`)
- Aliases (stored in `.rustshell_aliases`)
- Keyboard shortcuts (Ctrl+C to exit, Ctrl+A to move to start of line, etc.)

### Alias Management

Create and use aliases to save typing common commands:

```
# Create an alias
rustshell alias ll list -la

# Use the alias
rustshell ll

# List all aliases
rustshell alias

# Remove an alias
rustshell unalias ll
```

### Command Piping

Connect commands together in pipelines:

```
rustshell pipe 'list' 'grep txt'
```

This will list all files and then filter for ones containing "txt".

## Future Enhancements

- More advanced commands
- Support for redirections (>, >>, <)
- Custom scripting capabilities
- Plugin support 