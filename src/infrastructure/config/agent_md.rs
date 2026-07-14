use std::fs;
use std::path::{Path, PathBuf};

use noyalib::compat::serde_yaml;

use crate::application::config::{AgentDef, AgentSettings};
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

fn load_one(user_dir: &Path, repo_dir: &Path, name: &str) -> Result<AgentDef, MuonError> {
    for dir in [user_dir, repo_dir] {
        match load_by_name(dir, name)? {
            Some(def) => return Ok(def),
            None => {}
        }
    }
    Ok(AgentDef {
        name: name.to_string(),
        ..AgentDef::default()
    })
}

/// Load the six agent definitions (user `agents_dir` first, then bundled examples).
pub fn load_agent_settings(agents_dir: &Path) -> Result<AgentSettings, MuonError> {
    let repo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agents");
    Ok(AgentSettings {
        intent_classifier: load_one(agents_dir, &repo_dir, "intent-classifier")?,
        clarifier: load_one(agents_dir, &repo_dir, "clarifier")?,
        shallow_researcher: load_one(agents_dir, &repo_dir, "shallow-researcher")?,
        deep_orchestrator: load_one(agents_dir, &repo_dir, "deep-orchestrator")?,
        planner: load_one(agents_dir, &repo_dir, "planner")?,
        researcher: load_one(agents_dir, &repo_dir, "researcher")?,
    })
}

/// Write one agent file: YAML frontmatter + preserved markdown body.
pub fn save_agent_md(path: &Path, def: &AgentDef) -> Result<(), MuonError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            MuonError::Config(format!("agent file {path:?}: mkdir failed: {e}"))
        })?;
    }
    let yaml = serde_yaml::to_string(def)
        .map_err(|e| MuonError::Config(format!("agent file {path:?}: serialize YAML: {e}")))?;
    let yaml = yaml
        .strip_prefix("---\n")
        .or_else(|| yaml.strip_prefix("---\r\n"))
        .unwrap_or(&yaml)
        .trim_end()
        .to_string();
    let body = def.preamble_markdown.trim();
    let content = if body.is_empty() {
        format!("---\n{yaml}\n---\n")
    } else {
        format!("---\n{yaml}\n---\n\n{body}\n")
    };
    fs::write(path, content)
        .map_err(|e| MuonError::Config(format!("agent file {path:?}: write failed: {e}")))?;
    Ok(())
}

/// Persist all six agent definitions into `agents_dir` (YAML frontmatter SSOT).
pub fn save_agent_settings(agents_dir: &Path, settings: &AgentSettings) -> Result<(), MuonError> {
    fs::create_dir_all(agents_dir).map_err(|e| {
        MuonError::Config(format!(
            "agents dir {}: mkdir failed: {e}",
            agents_dir.display()
        ))
    })?;
    for (name, def) in settings.iter() {
        let path = agents_dir.join(format!("{name}.md"));
        save_agent_md(&path, def)?;
    }
    Ok(())
}
