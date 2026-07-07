use rig_core::providers::openai;

use crate::config::ProviderConfig;
use crate::error::MuonError;

pub struct ResolvedClient {
    pub client: openai::Client,
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
        let client = openai::Client::builder()
            .api_key(&api_key)
            .base_url(&provider.base_url)
            .build()
            .map_err(|e| MuonError::Config(format!("failed to build client for '{}': {e}", provider.name)))?;
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
