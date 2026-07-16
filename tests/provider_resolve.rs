#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::config::{ProviderConfig, ProviderType};
use muon::domain::error::MuonError;
use muon::infrastructure::providers::ResolvedClient;

fn named_provider(name: &str) -> ProviderConfig {
    ProviderConfig {
        name: name.to_string(),
        base_url: "http://localhost".to_string(),
        api_key: String::new(),
        models: Vec::new(),
        provider_type: ProviderType::OpenAICompatible,
    }
}

#[test]
fn for_named_provider_missing_name_lists_available_and_points_at_config() {
    let providers = vec![named_provider("openai"), named_provider("anthropic")];
    let msg = match ResolvedClient::for_named_provider("gemini", &providers) {
        Err(MuonError::Config(s)) => s,
        Ok(_) => panic!("expected Config error, got Ok"),
        Err(other) => panic!("expected Config, got {other:?}"),
    };
    assert!(
        msg.contains("provider 'gemini' is not defined in [[providers]]"),
        "missing provider: {msg}"
    );
    assert!(
        msg.contains("Available [[providers]] names: openai, anthropic"),
        "available list: {msg}"
    );
    assert!(
        msg.contains("agents/*.md") && msg.contains("config.toml"),
        "should point at agent + config.toml: {msg}"
    );
    assert!(
        msg.contains("name = \"gemini\""),
        "should suggest adding entry: {msg}"
    );
}

#[test]
fn for_named_provider_empty_list_explains_none_configured() {
    let msg = match ResolvedClient::for_named_provider("openai", &[]) {
        Err(MuonError::Config(s)) => s,
        Ok(_) => panic!("expected Config error, got Ok"),
        Err(other) => panic!("expected Config, got {other:?}"),
    };
    assert!(
        msg.contains("none — add at least one [[providers]]"),
        "empty available: {msg}"
    );
}
