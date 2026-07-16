use std::path::PathBuf;

pub fn expand_tilde<P: Into<PathBuf>>(p: P) -> PathBuf {
    let path = p.into();
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix('~')
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest.trim_start_matches('/'));
    }
    path
}

pub fn http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(60))
        .build()
}

pub use crate::domain::extract_json;
