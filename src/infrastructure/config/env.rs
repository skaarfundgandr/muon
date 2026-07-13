use crate::application::config::ProviderConfig;
use crate::domain::error::MuonError;

pub fn expand_env(value: &str) -> Result<String, MuonError> {
    let trimmed = value.trim();
    if let Some(inner) = trimmed.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        let var = inner.trim();
        std::env::var(var).map_err(|_| {
            MuonError::Config(format!(
                "environment variable '{var}' not set (referenced by '${{{var}}}')"
            ))
        })
    } else {
        Ok(value.to_string())
    }
}

pub fn resolve_api_key(provider: &ProviderConfig) -> Result<String, MuonError> {
    expand_env(&provider.api_key)
}
