use super::{LLMConfig, LLMProvider, LLMProviderTrait, LLMRequest, LLMResponse};
use crate::llm::providers::{anthropic::AnthropicProvider, openai::OpenAIProvider};
use anyhow::{anyhow, Result};
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub enum LLMProviderEnum {
    OpenAI(OpenAIProvider),
    Anthropic(AnthropicProvider),
}

impl LLMProviderEnum {
    pub async fn generate(&self, request: &LLMRequest) -> Result<LLMResponse> {
        match self {
            LLMProviderEnum::OpenAI(provider) => provider.generate(request).await,
            LLMProviderEnum::Anthropic(provider) => provider.generate(request).await,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            LLMProviderEnum::OpenAI(provider) => provider.name(),
            LLMProviderEnum::Anthropic(provider) => provider.name(),
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            LLMProviderEnum::OpenAI(provider) => provider.is_available(),
            LLMProviderEnum::Anthropic(provider) => provider.is_available(),
        }
    }
}

pub struct LLMClient {
    provider: LLMProviderEnum,
    cache: Arc<Mutex<LruCache<u64, LLMResponse>>>,
    config: LLMConfig,
}

impl LLMClient {
    pub async fn new(config: LLMConfig) -> Result<Self> {
        let provider = match &config.provider {
            LLMProvider::OpenAI => {
                let api_key = config
                    .api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| anyhow!("OpenAI API key not found"))?;
                
                LLMProviderEnum::OpenAI(OpenAIProvider::new(api_key, config.model.clone())?)
            }
            LLMProvider::Anthropic => {
                let api_key = config
                    .api_key
                    .clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| anyhow!("Anthropic API key not found"))?;
                
                LLMProviderEnum::Anthropic(AnthropicProvider::new(api_key, config.model.clone())?)
            }
            LLMProvider::Local(endpoint) => {
                return Err(anyhow!("Local provider not yet implemented: {}", endpoint));
            }
            LLMProvider::Custom(endpoint) => {
                return Err(anyhow!("Custom provider not yet implemented: {}", endpoint));
            }
        };

        // Create cache with 100 entries
        let cache = Arc::new(Mutex::new(
            LruCache::new(NonZeroUsize::new(100).unwrap())
        ));

        Ok(Self {
            provider,
            cache,
            config,
        })
    }

    pub async fn generate(&self, request: &LLMRequest) -> Result<LLMResponse> {
        // Check cache first
        let cache_key = self.calculate_cache_key(request);
        
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached_response) = cache.get(&cache_key) {
                return Ok(cached_response.clone());
            }
        }

        // Generate new response
        let response = self.provider.generate(request).await?;

        // Cache the response
        {
            let mut cache = self.cache.lock().await;
            cache.put(cache_key, response.clone());
        }

        Ok(response)
    }

    pub fn is_available(&self) -> bool {
        self.provider.is_available()
    }

    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    pub fn config(&self) -> &LLMConfig {
        &self.config
    }

    fn calculate_cache_key(&self, request: &LLMRequest) -> u64 {
        let mut hasher = DefaultHasher::new();
        request.prompt.hash(&mut hasher);
        request.max_tokens.hash(&mut hasher);
        // Use integer representation of temperature for hashing
        ((request.temperature * 100.0) as u32).hash(&mut hasher);
        if let Some(ref context) = request.context {
            context.hash(&mut hasher);
        }
        self.config.model.hash(&mut hasher);
        hasher.finish()
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }

    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len()
    }
}