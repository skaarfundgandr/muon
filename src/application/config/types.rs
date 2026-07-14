use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MuonConfig {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub search: SearchConfig,
    pub agents: AgentsConfig,
    pub data_sources: DataSourcesConfig,
    pub display: DisplayConfig,
    pub advanced: AdvancedConfig,
    #[serde(default)]
    pub obsidian: ObsidianConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ProviderType {
    #[default]
    #[serde(rename = "openai_compatible")]
    OpenAICompatible,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "anthropic")]
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub models: Vec<ProviderModel>,
    #[serde(default)]
    pub provider_type: ProviderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    pub name: String,
    #[serde(default)]
    pub model_id: String,
    #[serde(default)]
    pub description: String,
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
    /// TOML key is `type` (SPEC / examples); `provider_type` accepted for older saves.
    #[serde(rename = "type", alias = "provider_type")]
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
    #[serde(default)]
    pub clarifier: ClarifierConfig,
    #[serde(default)]
    pub shallow_researcher: ShallowResearcherConfig,
    #[serde(default)]
    pub deep_researcher: DeepResearcherConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifierConfig {
    pub max_turns: u64,
    pub plan_approval: bool,
    pub max_iterations: u64,
}

impl Default for ClarifierConfig {
    fn default() -> Self {
        Self {
            max_turns: 3,
            plan_approval: true,
            max_iterations: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShallowResearcherConfig {
    pub max_llm_turns: u64,
    pub max_tool_iters: u64,
}

impl Default for ShallowResearcherConfig {
    fn default() -> Self {
        Self {
            max_llm_turns: 10,
            max_tool_iters: 5,
        }
    }
}

fn default_agents_dir() -> PathBuf {
    PathBuf::from("~/.config/muon/agents")
}

fn default_min_report_length() -> u64 {
    1000
}
fn default_min_report_sections() -> u64 {
    2
}
fn default_max_retries() -> u64 {
    3
}
fn default_planner_max_cycles() -> u64 {
    3
}
fn default_orch_max_tool_calls() -> u64 {
    2
}
fn default_planner_max_tool_calls() -> u64 {
    4
}
fn default_researcher_max_tool_calls() -> u64 {
    4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeepResearcherConfig {
    pub iterations: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u64,
    #[serde(default = "default_planner_max_cycles")]
    pub planner_max_cycles: u64,
    #[serde(default = "default_orch_max_tool_calls")]
    pub orchestrator_max_tool_calls: u64,
    #[serde(default = "default_planner_max_tool_calls")]
    pub planner_max_tool_calls: u64,
    #[serde(default = "default_researcher_max_tool_calls")]
    pub researcher_max_tool_calls: u64,
    pub citation_verify: bool,
    #[serde(default = "default_min_report_length")]
    pub min_report_length: u64,
    #[serde(default = "default_min_report_sections")]
    pub min_report_sections: u64,
}

impl Default for DeepResearcherConfig {
    fn default() -> Self {
        Self {
            iterations: 8,
            max_retries: default_max_retries(),
            planner_max_cycles: default_planner_max_cycles(),
            orchestrator_max_tool_calls: default_orch_max_tool_calls(),
            planner_max_tool_calls: default_planner_max_tool_calls(),
            researcher_max_tool_calls: default_researcher_max_tool_calls(),
            citation_verify: true,
            min_report_length: default_min_report_length(),
            min_report_sections: default_min_report_sections(),
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
    Vec::new()
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
            knowledge_layer_rag: false,
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
#[serde(default)]
pub struct AdvancedConfig {
    pub max_researcher_loops: u64,
    pub max_shallow_turns: u64,
    pub max_deep_turns: u64,
    pub escalate_agent: bool,
    pub compaction_threshold: f64,
    pub max_tool_calls_per_turn: u64,
    pub agent_preamble: String,
    #[serde(default = "default_agents_dir")]
    pub agents_dir: PathBuf,
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
            max_shallow_turns: 10,
            max_deep_turns: 25,
            escalate_agent: true,
            compaction_threshold: 0.80,
            max_tool_calls_per_turn: 50,
            agent_preamble: "You are \u{03BC}on, a deep research agent. Be extremely precise, \
                fact-check everything, compile structured summaries, and cite sources in full \
                formatting. Maintain terminal safety."
                .to_string(),
            agents_dir: default_agents_dir(),
            session_db_path: "~/.local/share/muon/sessions.db".to_string(),
            rag_db_path: "~/.local/share/muon/rag.db".to_string(),
            max_search_items: 15,
            embedding_model: "Xenova/bge-small-en-v1.5".to_string(),
            rag_top_k: 5,
            similarity_threshold: 0.70,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObsidianConfig {
    #[serde(default)]
    pub vault_path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub langsmith: LangSmithConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangSmithConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_ls_project")]
    pub project: String,
    #[serde(default = "default_ls_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_ls_service")]
    pub service_name: String,
    #[serde(default)]
    pub console: bool,
    #[serde(default = "default_true")]
    pub batch: bool,
    #[serde(default = "default_batch_delay")]
    pub batch_delay_ms: u64,
}

impl Default for LangSmithConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            project: default_ls_project(),
            endpoint: default_ls_endpoint(),
            service_name: default_ls_service(),
            console: false,
            batch: default_true(),
            batch_delay_ms: default_batch_delay(),
        }
    }
}

fn default_ls_project() -> String {
    "default".to_string()
}
fn default_ls_endpoint() -> String {
    "https://api.smith.langchain.com/otel/v1/traces".to_string()
}
fn default_ls_service() -> String {
    "muon".to_string()
}
fn default_true() -> bool {
    true
}
fn default_batch_delay() -> u64 {
    1000
}
