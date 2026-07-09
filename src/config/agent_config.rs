use std::fs;
use std::path::{Path, PathBuf};

use noyalib::compat::serde_yaml;
use serde::Deserialize;

use crate::error::MuonError;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentDef {
    pub name: String,
    pub model: String,
    pub provider: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(skip)]
    pub preamble_markdown: String,
}

fn default_temperature() -> f64 {
    0.0
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_timeout_secs() -> u64 {
    60
}

pub fn parse_agent_md(path: &Path) -> Result<AgentDef, MuonError> {
    let text = fs::read_to_string(path)
        .map_err(|e| MuonError::Config(format!("agent file {path:?}: read failed: {e}")))?;
    if !text.starts_with("---") {
        return Err(MuonError::Config(format!(
            "agent file {path:?}: missing opening frontmatter delimiter"
        )));
    }
    let mut parts = text.splitn(2, "\n---\n");
    let header = parts
        .next()
        .ok_or_else(|| MuonError::Config(format!("agent file {path:?}: missing frontmatter")))?;
    let body = parts.next().ok_or_else(|| {
        MuonError::Config(format!(
            "agent file {path:?}: missing closing delimiter or body"
        ))
    })?;
    let yaml = header
        .strip_prefix("---\n")
        .or_else(|| header.strip_prefix("---"))
        .unwrap_or(header);
    let mut def: AgentDef = serde_yaml::from_str(yaml)
        .map_err(|e| MuonError::Config(format!("agent file {path:?}: invalid YAML: {e}")))?;
    def.preamble_markdown = body.trim().to_string();
    Ok(def)
}

pub fn load_by_name(dir: &Path, name: &str) -> Result<Option<AgentDef>, MuonError> {
    let path: PathBuf = dir.join(format!("{name}.md"));
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(parse_agent_md(&path)?))
}
