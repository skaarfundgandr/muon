use std::path::PathBuf;
use std::time::Duration;

use futures::Stream;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

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
        toml::from_str(&content).unwrap_or_default()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourcesConfig {
    pub web_search: bool,
    pub paper_search: bool,
    pub enterprise_systems: bool,
    pub knowledge_layer_rag: bool,
}

impl Default for DataSourcesConfig {
    fn default() -> Self {
        Self {
            web_search: true,
            paper_search: true,
            enterprise_systems: false,
            knowledge_layer_rag: true,
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
