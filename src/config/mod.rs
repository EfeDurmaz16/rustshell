use crate::llm::{LLMConfig, LLMProvider};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustShellConfig {
    pub llm: LLMSettings,
    pub safety: SafetySettings,
    pub features: FeatureSettings,
    pub ui: UISettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSettings {
    pub provider: String,
    pub model: String,
    pub api_key_env: Option<String>,
    pub endpoint: Option<String>,
    pub timeout_seconds: u64,
    pub max_tokens: u32,
    pub temperature: f32,
    pub enable_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySettings {
    pub require_confirmation: Vec<String>,
    pub dangerous_patterns: Vec<String>,
    pub enable_dry_run: bool,
    pub block_destructive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSettings {
    pub enable_llm: bool,
    pub fallback_to_traditional: bool,
    pub offline_mode: bool,
    pub enable_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub show_hints: bool,
    pub colored_output: bool,
    pub verbose_mode: bool,
    pub confirm_destructive: bool,
}

impl Default for RustShellConfig {
    fn default() -> Self {
        Self {
            llm: LLMSettings {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                endpoint: None,
                timeout_seconds: 30,
                max_tokens: 150,
                temperature: 0.1,
                enable_cache: true,
            },
            safety: SafetySettings {
                require_confirmation: vec![
                    "rm".to_string(),
                    "delete".to_string(),
                    "format".to_string(),
                    "sudo".to_string(),
                    "rmdir".to_string(),
                ],
                dangerous_patterns: vec![
                    "rm -rf /".to_string(),
                    "format c:".to_string(),
                    "sudo rm".to_string(),
                    "del /s".to_string(),
                ],
                enable_dry_run: true,
                block_destructive: false,
            },
            features: FeatureSettings {
                enable_llm: true,
                fallback_to_traditional: true,
                offline_mode: false,
                enable_history: true,
            },
            ui: UISettings {
                show_hints: true,
                colored_output: true,
                verbose_mode: false,
                confirm_destructive: true,
            },
        }
    }
}

impl RustShellConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: RustShellConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    pub fn to_llm_config(&self) -> Result<LLMConfig> {
        let provider = match self.llm.provider.as_str() {
            "openai" => LLMProvider::OpenAI,
            "anthropic" => LLMProvider::Anthropic,
            provider_str if provider_str.starts_with("local:") => {
                let endpoint = provider_str.strip_prefix("local:").unwrap().to_string();
                LLMProvider::Local(endpoint)
            }
            provider_str if provider_str.starts_with("custom:") => {
                let endpoint = provider_str.strip_prefix("custom:").unwrap().to_string();
                LLMProvider::Custom(endpoint)
            }
            _ => return Err(anyhow::anyhow!("Unknown LLM provider: {}", self.llm.provider)),
        };

        // Try multiple sources for API key:
        // 1. Environment variable (if api_key_env is set)
        // 2. Direct api_key field in config
        // 3. Default environment variables based on provider
        let api_key = if let Some(env_var) = &self.llm.api_key_env {
            // If it looks like an actual key (starts with sk-), use it directly
            if env_var.starts_with("sk-") || env_var.starts_with("anthropic-") {
                Some(env_var.clone())
            } else {
                // Otherwise treat it as an environment variable name
                std::env::var(env_var).ok()
            }
        } else {
            // Try default environment variables based on provider
            match provider {
                LLMProvider::OpenAI => std::env::var("OPENAI_API_KEY").ok(),
                LLMProvider::Anthropic => std::env::var("ANTHROPIC_API_KEY").ok(),
                _ => None,
            }
        };

        Ok(LLMConfig {
            provider,
            model: self.llm.model.clone(),
            api_key,
            endpoint: self.llm.endpoint.clone(),
            timeout: Duration::from_secs(self.llm.timeout_seconds),
            max_tokens: self.llm.max_tokens,
            temperature: self.llm.temperature,
        })
    }

    fn config_file_path() -> Result<PathBuf> {
        let home_dir = dirs_next::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        
        Ok(home_dir.join(".rustshell").join("config.toml"))
    }

    pub fn is_dangerous_command(&self, command: &str) -> bool {
        let command_lower = command.to_lowercase();
        
        // Check dangerous patterns
        for pattern in &self.safety.dangerous_patterns {
            if command_lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }
        
        false
    }

    pub fn requires_confirmation(&self, command: &str) -> bool {
        if !self.ui.confirm_destructive {
            return false;
        }
        
        let command_lower = command.to_lowercase();
        
        // Check if command requires confirmation
        for confirm_cmd in &self.safety.require_confirmation {
            if command_lower.starts_with(&confirm_cmd.to_lowercase()) {
                return true;
            }
        }
        
        false
    }
}

pub fn get_config() -> Result<RustShellConfig> {
    RustShellConfig::load()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RustShellConfig::default();
        assert!(config.features.enable_llm);
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_dangerous_command_detection() {
        let config = RustShellConfig::default();
        
        assert!(config.is_dangerous_command("rm -rf /"));
        assert!(config.is_dangerous_command("sudo rm -rf /home"));
        assert!(!config.is_dangerous_command("ls -la"));
        assert!(!config.is_dangerous_command("mkdir test"));
    }

    #[test]
    fn test_confirmation_required() {
        let config = RustShellConfig::default();
        
        assert!(config.requires_confirmation("rm file.txt"));
        assert!(config.requires_confirmation("delete directory"));
        assert!(!config.requires_confirmation("ls -la"));
        assert!(!config.requires_confirmation("mkdir test"));
    }
}