use std::path::PathBuf;
use std::time::Duration;

use futures::Stream;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::MuonError;

fn config_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let mut p = PathBuf::from(home);
    p.push(".config");
    p.push("muon");
    Some(p)
}

fn config_path() -> Option<PathBuf> {
    let mut dir = config_dir()?;
    dir.push("config.toml");
    Some(dir)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MuonConfig {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub search: SearchConfig,
    pub agents: AgentsConfig,
    pub tools: ToolsConfig,
    pub data_sources: DataSourcesConfig,
    pub display: DisplayConfig,
    pub advanced: AdvancedConfig,
}

impl MuonConfig {
    pub fn load() -> Self {
        let path = match config_path() {
            Some(p) => p,
            None => return Self::default(),
        };
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &std::path::Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        let mut cfg: Self = match toml::from_str(&content) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        cfg.backfill_legacy_providers();
        cfg
    }

    pub fn watch() -> impl Stream<Item = MuonConfig> {
        let dir = config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::watch_inner(dir)
    }

    pub fn watch_path(path: PathBuf) -> impl Stream<Item = MuonConfig> {
        Self::watch_inner(path)
    }

    fn watch_inner(dir: PathBuf) -> impl Stream<Item = MuonConfig> {
        let (signal_tx, mut signal_rx) = mpsc::channel::<()>(8);
        let (config_tx, config_rx) = mpsc::channel::<MuonConfig>(4);

        let watch_dir = dir.clone();
        std::thread::Builder::new()
            .name("config-watcher".to_string())
            .spawn(move || {
                let mut watcher: RecommendedWatcher =
                    match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                        let Ok(event) = res else {
                            return;
                        };
                        match event.kind {
                            notify::EventKind::Create(_)
                            | notify::EventKind::Modify(_) => {
                                let _ = signal_tx.blocking_send(());
                            }
                            _ => {}
                        }
                    }) {
                        Ok(w) => w,
                        Err(_) => return,
                    };

                if watcher.watch(&watch_dir, RecursiveMode::NonRecursive).is_err() {
                    return;
                }

                loop {
                    std::thread::sleep(Duration::from_secs(3600));
                }
            })
            .ok();

        let config_file = dir.join("config.toml");
        tokio::spawn(async move {
            while let Some(()) = signal_rx.recv().await {
                tokio::time::sleep(Duration::from_millis(300)).await;
                while signal_rx.try_recv().is_ok() {}

                let config = MuonConfig::load_from_path(&config_file);
                if config_tx.send(config).await.is_err() {
                    break;
                }
            }
        });

        let mut inner_rx = config_rx;
        futures::stream::poll_fn(move |cx| inner_rx.poll_recv(cx))
    }

    pub fn save(&self) {
        let path = match config_path() {
            Some(p) => p,
            None => return,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = match toml::to_string_pretty(self) {
            Ok(c) => c,
            Err(_) => return,
        };
        let _ = std::fs::write(&path, content);
    }

    pub fn backfill_legacy_providers(&mut self) {
        if !self.providers.is_empty() {
            return;
        }
        let mk = |name: &str, base_url: String, key: String| ProviderConfig {
            name: name.to_string(),
            base_url,
            api_key: key,
            models: Vec::new(),
        };
        if !self.tools.opencode_go_api_key.is_empty() {
            self.providers.push(mk(
                "opencode-go",
                std::env::var("OPENCODE_GO_BASE_URL")
                    .unwrap_or_else(|_| "https://api.opencode.dev/v1".into()),
                self.tools.opencode_go_api_key.clone(),
            ));
        }
        if !self.tools.neuralwatt_api_key.is_empty() {
            self.providers.push(mk(
                "NeuralWatt",
                std::env::var("NEURALWATT_BASE_URL")
                    .unwrap_or_else(|_| "https://api.neuralwatt.com/v1".into()),
                self.tools.neuralwatt_api_key.clone(),
            ));
        }
        if !self.tools.clinepass_api_key.is_empty() {
            self.providers.push(mk(
                "cline",
                std::env::var("CLINEPASS_BASE_URL")
                    .unwrap_or_else(|_| "https://api.cline.bot/v1".into()),
                self.tools.clinepass_api_key.clone(),
            ));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub models: Vec<ProviderModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    pub name: String,
    #[serde(default)]
    pub model_id: String,
    #[serde(default)]
    pub description: String,
}

impl ProviderConfig {
    pub fn resolved_api_key(&self) -> Result<String, MuonError> {
        expand_env(&self.api_key)
    }
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default)]
    pub providers: Vec<SearchProviderConfig>,
    #[serde(default)]
    pub papers: PapersConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PapersConfig {
    #[serde(default = "default_arxiv_enabled")]
    pub arxiv_enabled: bool,
}

fn default_arxiv_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchProviderConfig {
    pub name: String,
    pub provider_type: SearchProviderType,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub max_results: Option<usize>,
    #[serde(default)]
    pub tavily: TavilyOptions,
    #[serde(default)]
    pub firecrawl: FirecrawlOptions,
    #[serde(default)]
    pub brave: BraveOptions,
    #[serde(default)]
    pub serper: SerperOptions,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SearchProviderType {
    #[default]
    Tavily,
    Firecrawl,
    Brave,
    Serper,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TavilyOptions {
    #[serde(default)]
    pub search_depth: TavilySearchDepth,
    #[serde(default)]
    pub include_answer: bool,
    #[serde(default)]
    pub include_raw_content: bool,
    #[serde(default)]
    pub topic: Option<TavilyTopic>,
    #[serde(default)]
    pub time_range: Option<String>,
    #[serde(default)]
    pub include_domains: Vec<String>,
    #[serde(default)]
    pub exclude_domains: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TavilySearchDepth {
    #[default]
    Basic,
    Fast,
    UltraFast,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TavilyTopic {
    General,
    News,
    Finance,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FirecrawlOptions {
    #[serde(default = "default_firecrawl_formats")]
    pub scrape_formats: Vec<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub include_domains: Vec<String>,
    #[serde(default)]
    pub exclude_domains: Vec<String>,
    #[serde(default)]
    pub categories: Vec<FirecrawlCategory>,
}

fn default_firecrawl_formats() -> Vec<String> {
    vec!["markdown".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirecrawlCategory {
    Github,
    Research,
    Pdf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BraveOptions {
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub search_lang: Option<String>,
    #[serde(default)]
    pub extra_snippets: bool,
    #[serde(default)]
    pub goggles_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerperOptions {
    #[serde(default)]
    pub gl: Option<String>,
    #[serde(default)]
    pub hl: Option<String>,
    #[serde(default)]
    pub tbs: Option<String>,
    #[serde(default)]
    pub autocorrect: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub intent_classifier: AgentEntryConfig,
    pub clarifier: ClarifierConfig,
    pub shallow_researcher: ShallowResearcherConfig,
    pub deep_researcher: DeepResearcherConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntryConfig {
    pub model: String,
    pub provider: String,
    pub timeout_sec: u64,
    pub verbose: bool,
}

impl Default for AgentEntryConfig {
    fn default() -> Self {
        Self {
            model: "glm-5.2".to_string(),
            provider: "opencode-go".to_string(),
            timeout_sec: 90,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifierConfig {
    pub model: String,
    pub provider: String,
    pub max_turns: u64,
    pub plan_approval: bool,
    pub max_iterations: u64,
}

impl Default for ClarifierConfig {
    fn default() -> Self {
        Self {
            model: "glm-5.2".to_string(),
            provider: "opencode-go".to_string(),
            max_turns: 3,
            plan_approval: true,
            max_iterations: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShallowResearcherConfig {
    pub model: String,
    pub provider: String,
    pub max_llm_turns: u64,
    pub max_tool_iters: u64,
}

impl Default for ShallowResearcherConfig {
    fn default() -> Self {
        Self {
            model: "glm-5.2".to_string(),
            provider: "NeuralWatt".to_string(),
            max_llm_turns: 10,
            max_tool_iters: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearcherConfig {
    pub orchestrator: SubagentConfig,
    pub planner: SubagentConfig,
    pub researcher: SubagentConfig,
    pub iterations: u64,
    pub citation_verify: bool,
}

impl Default for DeepResearcherConfig {
    fn default() -> Self {
        Self {
            orchestrator: SubagentConfig {
                model: "glm-5.2".to_string(),
                provider: "opencode-go".to_string(),
            },
            planner: SubagentConfig {
                model: "glm-5.2-short".to_string(),
                provider: "NeuralWatt".to_string(),
            },
            researcher: SubagentConfig {
                model: "glm-5.2-flex".to_string(),
                provider: "NeuralWatt".to_string(),
            },
            iterations: 2,
            citation_verify: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    pub model: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub opencode_go_api_key: String,
    pub neuralwatt_api_key: String,
    pub clinepass_api_key: String,
    pub brave_api_key: String,
    pub searxng_url: String,
    pub searxng_api_key: String,
    pub semantic_scholar_api_key: String,
    pub arxiv_enabled: bool,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            opencode_go_api_key: String::new(),
            neuralwatt_api_key: String::new(),
            clinepass_api_key: String::new(),
            brave_api_key: String::new(),
            searxng_url: "https://searxng.local".to_string(),
            searxng_api_key: String::new(),
            semantic_scholar_api_key: String::new(),
            arxiv_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RagIndexConfig {
    pub path: String,
    pub kind: String,
    pub status: String,
    pub chunks: String,
}

fn default_rag_indexes() -> Vec<RagIndexConfig> {
    vec![
        RagIndexConfig {
            path: "~/Documents/research/".to_string(),
            kind: "DIR".to_string(),
            status: "✓ indexed".to_string(),
            chunks: "2,841".to_string(),
        },
        RagIndexConfig {
            path: "~/.muon/notes/".to_string(),
            kind: "DIR".to_string(),
            status: "✓ indexed".to_string(),
            chunks: "412".to_string(),
        },
        RagIndexConfig {
            path: "papers/*.pdf".to_string(),
            kind: "GLOB".to_string(),
            status: "◉ indexing".to_string(),
            chunks: "1,209".to_string(),
        },
        RagIndexConfig {
            path: "README.md".to_string(),
            kind: "FILE".to_string(),
            status: "○ pending".to_string(),
            chunks: "0".to_string(),
        },
        RagIndexConfig {
            path: "~/.muon/sources.csv".to_string(),
            kind: "FILE".to_string(),
            status: "✓ indexed".to_string(),
            chunks: "89".to_string(),
        },
    ]
}

fn default_source_path() -> String {
    "~/documents/research/".to_string()
}

fn default_source_type() -> String {
    "Directory".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourcesConfig {
    pub web_search: bool,
    pub paper_search: bool,
    pub enterprise_systems: bool,
    pub knowledge_layer_rag: bool,
    #[serde(default = "default_rag_indexes")]
    pub rag_indexes: Vec<RagIndexConfig>,
    #[serde(skip, default = "default_source_path")]
    pub source_path: String,
    #[serde(skip, default = "default_source_type")]
    pub source_type: String,
}

impl Default for DataSourcesConfig {
    fn default() -> Self {
        Self {
            web_search: true,
            paper_search: true,
            enterprise_systems: false,
            knowledge_layer_rag: true,
            rag_indexes: default_rag_indexes(),
            source_path: default_source_path(),
            source_type: default_source_type(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub visual_theme: String,
    pub font_size: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            visual_theme: "Tokyo Night".to_string(),
            font_size: "Medium 14px".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub max_researcher_loops: u64,
    pub max_clarifier_turns: u64,
    pub max_plan_iterations: u64,
    pub max_shallow_turns: u64,
    pub max_deep_turns: u64,
    pub escalate_agent: bool,
    pub plan_approval: bool,
    pub compaction_threshold: f64,
    pub agent_preamble: String,
    pub session_db_path: String,
    pub rag_db_path: String,
    pub max_search_items: u64,
    pub embedding_model: String,
    pub rag_top_k: u64,
    pub similarity_threshold: f64,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            max_researcher_loops: 2,
            max_clarifier_turns: 3,
            max_plan_iterations: 10,
            max_shallow_turns: 10,
            max_deep_turns: 25,
            escalate_agent: true,
            plan_approval: true,
            compaction_threshold: 0.80,
            agent_preamble: "You are \u{03BC}on, a deep research agent. Be extremely precise, \
                fact-check everything, compile structured summaries, and cite sources in full \
                formatting. Maintain terminal safety."
                .to_string(),
            session_db_path: "~/.local/share/muon/sessions.db".to_string(),
            rag_db_path: "~/.local/share/muon/rag.db".to_string(),
            max_search_items: 15,
            embedding_model: "Xenova/bge-small-en-v1.5".to_string(),
            rag_top_k: 5,
            similarity_threshold: 0.70,
        }
    }
}
