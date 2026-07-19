use std::path::PathBuf;
use std::time::Duration;

use futures::Stream;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::application::config::MuonConfig;
use crate::domain::error::MuonError;

pub fn config_dir() -> Option<PathBuf> {
    let mut p = dirs::home_dir()?;
    p.push(".config");
    p.push("muon");
    Some(p)
}

fn config_path() -> Option<PathBuf> {
    let mut dir = config_dir()?;
    dir.push("config.toml");
    Some(dir)
}

pub fn load() -> MuonConfig {
    let path = match config_path() {
        Some(p) => p,
        None => return MuonConfig::default(),
    };
    load_from_path(&path)
}

pub fn load_from_path(path: &std::path::Path) -> MuonConfig {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return MuonConfig::default(),
    };
    match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(
                target: "muon::config",
                path = %path.display(),
                error = %e,
                "failed to parse config.toml; using defaults"
            );
            MuonConfig::default()
        }
    }
}

pub fn save(cfg: &MuonConfig) -> Result<(), MuonError> {
    let path = config_path()
        .ok_or_else(|| MuonError::Config("cannot resolve config path (home directory unknown)".into()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let copy = cfg.clone();
    let content = toml::to_string_pretty(&copy)
        .map_err(|e| MuonError::Config(format!("serialize config: {e}")))?;
    std::fs::write(&path, &content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms)?;
    }
    Ok(())
}

pub fn watch() -> impl Stream<Item = MuonConfig> {
    let dir = config_dir().unwrap_or_else(|| PathBuf::from("."));
    watch_inner(dir)
}

pub fn watch_path(path: PathBuf) -> impl Stream<Item = MuonConfig> {
    watch_inner(path)
}

fn watch_inner(dir: PathBuf) -> impl Stream<Item = MuonConfig> {
    let (signal_tx, mut signal_rx) = mpsc::channel::<()>(8);
    let (config_tx, config_rx) = mpsc::channel::<MuonConfig>(4);

    let watch_dir = dir.clone();
    std::thread::Builder::new()
        .name("config-watcher".to_string())
        .spawn(move || {
            let mut watcher: RecommendedWatcher = match notify::recommended_watcher(
                move |res: Result<notify::Event, notify::Error>| {
                    let Ok(event) = res else {
                        return;
                    };
                    match event.kind {
                        notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                            let _ = signal_tx.blocking_send(());
                        }
                        _ => {}
                    }
                },
            ) {
                Ok(w) => w,
                Err(_) => return,
            };

            if watcher
                .watch(&watch_dir, RecursiveMode::NonRecursive)
                .is_err()
            {
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

            let config = load_from_path(&config_file);
            if config_tx.send(config).await.is_err() {
                break;
            }
        }
    });

    let mut inner_rx = config_rx;
    futures::stream::poll_fn(move |cx| inner_rx.poll_recv(cx))
}

pub fn ensure_scaffolded() {
    let Some(dir) = config_dir() else { return };
    ensure_scaffolded_in(&dir);
}

pub fn ensure_scaffolded_in(config_dir: &std::path::Path) {
    let cfg_path = config_dir.join("config.toml");
    if !cfg_path.exists() {
        if let Err(e) = std::fs::create_dir_all(config_dir) {
            tracing::warn!(target: "muon::config", "scaffold: mkdir failed: {e}");
            return;
        }
        let content = include_str!("../../../examples/muon.scaffold.toml");
        if let Err(e) = std::fs::write(&cfg_path, content) {
            tracing::warn!(target: "muon::config", "scaffold: config.toml write failed: {e}");
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&cfg_path, std::fs::Permissions::from_mode(0o600));
        }
    }
    let agents_dir = config_dir.join("agents");
    if std::fs::create_dir_all(&agents_dir).is_err() {
        return;
    }
    for (name, content) in SCAFFOLD_AGENT_FILES {
        let path = agents_dir.join(name);
        if path.exists() {
            continue;
        }
        if let Err(e) = std::fs::write(&path, content) {
            tracing::warn!(target: "muon::config", "scaffold: agent '{name}' write failed: {e}");
        }
    }
}

const SCAFFOLD_AGENT_FILES: &[(&str, &str)] = &[
    (
        "intent-classifier.md",
        include_str!("../../../examples/agents/intent-classifier.md"),
    ),
    (
        "clarifier.md",
        include_str!("../../../examples/agents/clarifier.md"),
    ),
    (
        "shallow-researcher.md",
        include_str!("../../../examples/agents/shallow-researcher.md"),
    ),
    (
        "deep-orchestrator.md",
        include_str!("../../../examples/agents/deep-orchestrator.md"),
    ),
    (
        "planner.md",
        include_str!("../../../examples/agents/planner.md"),
    ),
    (
        "researcher.md",
        include_str!("../../../examples/agents/researcher.md"),
    ),
];
