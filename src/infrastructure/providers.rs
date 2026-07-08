use rig_core::providers::{anthropic, gemini, openai};

use crate::config::{ProviderConfig, ProviderType};
use crate::error::MuonError;

pub enum ProviderClient {
    OpenAI(openai::Client),
    OpenAICompatible(openai::Client),
    Gemini(gemini::Client),
    Anthropic(anthropic::Client),
}

pub struct ResolvedClient {
    pub client: ProviderClient,
    pub provider_name: String,
}

impl ResolvedClient {
    pub fn for_named_provider(
        name: &str,
        providers: &[ProviderConfig],
    ) -> Result<Self, MuonError> {
        let provider = providers
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| MuonError::Config(format!("provider '{name}' not found in [[providers]]")))?;
        Self::for_provider_config(provider)
    }

    pub fn for_provider_config(provider: &ProviderConfig) -> Result<Self, MuonError> {
        let api_key = provider.resolved_api_key()?;
        let client = match provider.provider_type {
            ProviderType::OpenAI | ProviderType::OpenAICompatible => {
                let c = openai::Client::builder()
                    .api_key(&api_key)
                    .base_url(&provider.base_url)
                    .build()
                    .map_err(|e| MuonError::Config(format!("failed to build client for '{}': {e}", provider.name)))?;
                if provider.provider_type == ProviderType::OpenAI {
                    ProviderClient::OpenAI(c)
                } else {
                    ProviderClient::OpenAICompatible(c)
                }
            }
            ProviderType::Gemini => {
                let c = gemini::Client::builder()
                    .api_key(&api_key)
                    .base_url(&provider.base_url)
                    .build()
                    .map_err(|e| MuonError::Config(format!("failed to build gemini client for '{}': {e}", provider.name)))?;
                ProviderClient::Gemini(c)
            }
            ProviderType::Anthropic => {
                let c = anthropic::Client::builder()
                    .api_key(&api_key)
                    .base_url(&provider.base_url)
                    .build()
                    .map_err(|e| MuonError::Config(format!("failed to build anthropic client for '{}': {e}", provider.name)))?;
                ProviderClient::Anthropic(c)
            }
        };
        Ok(Self {
            client,
            provider_name: provider.name.clone(),
        })
    }

    pub fn for_default_provider(providers: &[ProviderConfig]) -> Result<Self, MuonError> {
        let provider = providers
            .first()
            .ok_or_else(|| MuonError::Config("no [[providers]] configured — add at least one".into()))?;
        Self::for_provider_config(provider)
    }
}
