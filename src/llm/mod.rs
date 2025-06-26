use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod client;
pub mod prompts;
pub mod providers;

pub use client::LLMClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Local(String),
    Custom(String),
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProvider::OpenAI => write!(f, "OpenAI"),
            LLMProvider::Anthropic => write!(f, "Anthropic"),
            LLMProvider::Local(endpoint) => write!(f, "Local({})", endpoint),
            LLMProvider::Custom(endpoint) => write!(f, "Custom({})", endpoint),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub timeout: Duration,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            endpoint: None,
            timeout: Duration::from_secs(30),
            max_tokens: 150,
            temperature: 0.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub finish_reason: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait::async_trait]
pub trait LLMProviderTrait: Send + Sync {
    async fn generate(&self, request: &LLMRequest) -> Result<LLMResponse>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
}