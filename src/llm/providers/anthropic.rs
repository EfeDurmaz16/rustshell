use crate::llm::{LLMProviderTrait, LLMRequest, LLMResponse, Usage};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model,
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
        })
    }

    pub fn with_endpoint(api_key: String, model: String, endpoint: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model,
            endpoint,
        })
    }
}

#[async_trait::async_trait]
impl LLMProviderTrait for AnthropicProvider {
    async fn generate(&self, request: &LLMRequest) -> Result<LLMResponse> {
        let mut messages = Vec::new();
        
        // Add system message if context is provided
        if let Some(context) = &request.context {
            messages.push(AnthropicMessage {
                role: "system".to_string(),
                content: context.clone(),
            });
        }
        
        messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            messages,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic API error {}: {}", status, text));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;

        let content = anthropic_response
            .content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join(" ");

        let usage = anthropic_response.usage.map(|u| Usage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        Ok(LLMResponse {
            content: content.trim().to_string(),
            finish_reason: anthropic_response.stop_reason.unwrap_or_default(),
            usage,
        })
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}