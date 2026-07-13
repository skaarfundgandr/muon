use std::fs;
use std::path::{Path, PathBuf};

use noyalib::compat::serde_yaml;

use crate::application::config::AgentDef;
use crate::domain::error::MuonError;

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
