pub mod paper_search;
pub mod provider;
pub mod web_search;

pub use paper_search::{ArxivProvider, SemanticScholarProvider};
pub use provider::{resolve_paper_providers, resolve_web_provider, WebProviderKind};
pub use web_search::{BraveProvider, SearXngProvider};
