pub mod composite;
pub mod paper_search;
pub mod provider;
pub mod web_search;

pub use composite::CompositeSearchProvider;
pub(crate) use composite::percent_encode;
pub use paper_search::ArxivProvider;
pub use provider::{resolve_paper_providers, resolve_web_provider};
pub use web_search::{BraveProvider, FirecrawlProvider, SerperProvider, TavilyProvider};
