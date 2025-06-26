use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub system_prompt: String,
    pub os_context: HashMap<String, String>,
    pub safety_rules: Vec<String>,
    pub output_format: String,
}

impl PromptTemplate {
    pub fn new() -> Self {
        let mut os_context = HashMap::new();
        os_context.insert("windows".to_string(), "Windows PowerShell/Command Prompt".to_string());
        os_context.insert("linux".to_string(), "Linux bash/sh shell".to_string());
        os_context.insert("macos".to_string(), "macOS bash/zsh shell".to_string());

        let safety_rules = vec![
            "Never suggest commands that could harm the system".to_string(),
            "Always use safe file operations".to_string(),
            "Warn about destructive operations".to_string(),
            "Prefer native cross-platform commands when possible".to_string(),
            "Only return the command, no explanations".to_string(),
        ];

        Self {
            system_prompt: Self::default_system_prompt(),
            os_context,
            safety_rules,
            output_format: Self::default_output_format(),
        }
    }

    pub fn build_prompt(&self, user_input: &str, os: &str) -> String {
        let default_os = "Unknown OS".to_string();
        let os_info = self.os_context.get(os).unwrap_or(&default_os);
        
        format!(
            "{}\n\nTarget OS: {}\nUser Request: \"{}\"\n\nSafety Rules:\n{}\n\nOutput Format:\n{}\n\nProvide only the command:",
            self.system_prompt,
            os_info,
            user_input,
            self.safety_rules.join("\n- "),
            self.output_format
        )
    }

    fn default_system_prompt() -> String {
        r#"You are a cross-platform command translator. Your job is to convert natural language requests into appropriate shell commands for the target operating system.

Key responsibilities:
1. Translate natural language to OS-specific commands
2. Ensure commands are safe and appropriate
3. Handle cross-platform differences (Windows vs Unix)
4. Provide only the command, no explanations

Examples:
- "create a directory called test" → "mkdir test" (Unix) or "mkdir test" (Windows)
- "list files in current directory" → "ls -la" (Unix) or "dir" (Windows)
- "copy file.txt to backup.txt" → "cp file.txt backup.txt" (Unix) or "copy file.txt backup.txt" (Windows)
- "delete file.txt" → "rm file.txt" (Unix) or "del file.txt" (Windows)
- "show current directory" → "pwd" (Unix) or "cd" (Windows)

Always consider the target OS and provide the most appropriate command."#.to_string()
    }

    fn default_output_format() -> String {
        r#"Return ONLY the command without any explanations, quotes, or additional text.
For compound operations, separate commands with ' && '.
For Windows, use PowerShell commands when appropriate.
For Unix systems, use standard shell commands."#.to_string()
    }
}

impl Default for PromptTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTranslation {
    pub original_request: String,
    pub translated_command: String,
    pub os: String,
    pub confidence: f32,
    pub safety_warnings: Vec<String>,
}

pub fn detect_os() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn is_natural_language(input: &str) -> bool {
    // Simple heuristics to detect natural language vs direct commands
    let natural_language_indicators = [
        "create", "make", "delete", "remove", "copy", "move", "show", "display",
        "list", "find", "search", "navigate", "go to", "change to", "switch to",
        "how to", "can you", "please", "i want", "i need", "help me",
    ];

    let input_lower = input.to_lowercase();
    
    // Check for natural language indicators
    for indicator in &natural_language_indicators {
        if input_lower.contains(indicator) {
            return true;
        }
    }

    // Check for complete sentences (contains multiple words and common sentence patterns)
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() > 3 {
        // Look for sentence-like patterns
        if input_lower.contains(" a ") || input_lower.contains(" an ") || 
           input_lower.contains(" the ") || input_lower.contains(" to ") ||
           input_lower.contains(" and ") || input_lower.contains(" or ") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_building() {
        let template = PromptTemplate::new();
        let prompt = template.build_prompt("create a directory called test", "linux");
        
        assert!(prompt.contains("create a directory called test"));
        assert!(prompt.contains("Linux bash/sh shell"));
        assert!(prompt.contains("Never suggest commands that could harm the system"));
    }

    #[test]
    fn test_natural_language_detection() {
        assert!(is_natural_language("create a new directory"));
        assert!(is_natural_language("please show me the files"));
        assert!(is_natural_language("I want to copy this file"));
        assert!(is_natural_language("how to delete a directory"));
        
        assert!(!is_natural_language("ls -la"));
        assert!(!is_natural_language("mkdir test"));
        assert!(!is_natural_language("cd /home"));
    }

    #[test]
    fn test_os_detection() {
        let os = detect_os();
        assert!(os == "windows" || os == "linux" || os == "macos");
    }
}