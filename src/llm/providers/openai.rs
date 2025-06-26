use crate::llm::{LLMProviderTrait, LLMRequest, LLMResponse, Usage};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug)]
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key,
            model,
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
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
impl LLMProviderTrait for OpenAIProvider {
    async fn generate(&self, request: &LLMRequest) -> Result<LLMResponse> {
        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: request.context.clone().unwrap_or_default(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            },
        ];

        let openai_request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API error {}: {}", status, text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let choice = openai_response
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No choices returned from OpenAI API"))?;

        let usage = openai_response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(LLMResponse {
            content: choice.message.content.trim().to_string(),
            finish_reason: choice.finish_reason.unwrap_or_default(),
            usage,
        })
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}