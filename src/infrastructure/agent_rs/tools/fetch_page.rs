use std::future::Future;
use std::net::IpAddr;

use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::error::MuonError;

const NAME: &str = "fetch_page";
const MAX_BODY_BYTES: usize = 2_000_000;

pub fn is_public_http_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|e| format!("invalid url: {e}"))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(format!("scheme not allowed: {other}")),
    }
    let host = url.host_str().ok_or_else(|| "missing host".to_string())?;
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return Err("localhost blocked".into());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(ip) {
            return Err("private or link-local address blocked".into());
        }
    } else if looks_like_ip_literal(host) {
        return Err("ip literal host blocked".into());
    }
    Ok(())
}

fn looks_like_ip_literal(host: &str) -> bool {
    host.chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ':')
        && host.contains(['.', ':'])
}

pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_multicast()
                || v4.octets()[0] == 169 && v4.octets()[1] == 254
                || v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 64
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6.is_unspecified()
                || v6.is_multicast()
        }
    }
}

pub async fn ensure_public_resolved(host: &str) -> Result<(), String> {
    let addrs = tokio::net::lookup_host((host, 0u16))
        .await
        .map_err(|e| format!("dns resolution failed: {e}"))?;
    let mut has_any = false;
    for addr in addrs {
        has_any = true;
        if is_blocked_ip(addr.ip()) {
            return Err(format!("resolved address {} is blocked", addr.ip()));
        }
    }
    if !has_any {
        return Err("dns resolution returned no addresses".into());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct FetchPageArgs {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct FetchPageOutput {
    pub url: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone)]
pub struct FetchPageTool {
    http: reqwest::Client,
    max_chars: usize,
}

impl FetchPageTool {
    pub fn new(max_chars: usize) -> Result<Self, MuonError> {
        let http = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(60))
            // Redirects intentionally disabled to prevent SSRF via unchecked Location headers.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| MuonError::Config(format!("failed to build http client: {e}")))?;
        Ok(Self { http, max_chars })
    }
}

impl Tool for FetchPageTool {
    const NAME: &'static str = NAME;
    type Error = MuonError;
    type Args = FetchPageArgs;
    type Output = FetchPageOutput;

    fn description(&self) -> String {
        "Fetch a web page and return its text content. HTML tags are stripped and content is truncated.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch." }
            },
            "required": ["url"]
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + rig_core::wasm_compat::WasmCompatSend
    {
        let http = self.http.clone();
        let max_chars = self.max_chars;
        async move {
            // First gate: fast hostname/IP check
            if let Err(reason) = is_public_http_url(&args.url) {
                return Err(MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("url blocked: {reason}"),
                });
            }

            // Second gate: DNS resolve-then-validate (guards against DNS rebinding)
            let url = Url::parse(&args.url).map_err(|e| MuonError::Search {
                provider: "fetch".into(),
                message: format!("invalid url: {e}"),
            })?;
            let host = url.host_str().ok_or_else(|| MuonError::Search {
                provider: "fetch".into(),
                message: "missing host".to_string(),
            })?;
            if let Err(reason) = ensure_public_resolved(host).await {
                return Err(MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("url blocked: {reason}"),
                });
            }

            let resp = http
                .get(&args.url)
                .send()
                .await
                .map_err(|e| MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("failed to fetch {}: {e}", args.url),
                })?;

            let status = resp.status();
            if !status.is_success() {
                return Err(MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("{} returned status {status}", args.url),
                });
            }

            let bytes = resp.bytes().await.map_err(|e| MuonError::Search {
                provider: "fetch".into(),
                message: format!("failed to read body from {}: {e}", args.url),
            })?;
            let capped = if bytes.len() > MAX_BODY_BYTES {
                &bytes[..MAX_BODY_BYTES]
            } else {
                &bytes
            };
            let html = String::from_utf8_lossy(capped).into_owned();

            let (text, title) = html_to_text(&html, max_chars);
            Ok(FetchPageOutput {
                url: args.url,
                text,
                title,
            })
        }
    }
}

fn html_to_text(html: &str, max_chars: usize) -> (String, Option<String>) {
    // Extract title from <title>...</title>
    let title = {
        let re = regex::Regex::new(r"(?is)<title[^>]*>(.*?)</title>").ok();
        re.and_then(|r| {
            r.captures(html)
                .and_then(|c| c.get(1))
                .map(|m| strip_tags_single(m.as_str()).trim().to_string())
        })
    };

    // Strip all HTML tags
    let tag_re = regex::Regex::new(r"(?s)<[^>]+>").ok();
    let mut text = match tag_re {
        Some(re) => re.replace_all(html, " ").to_string(),
        None => html.to_string(),
    };

    // Decode common HTML entities
    text = text.replace("&amp;", "&");
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&quot;", "\"");
    text = text.replace("&#39;", "'");
    text = text.replace("&nbsp;", " ");

    // Collapse whitespace
    let ws_re = regex::Regex::new(r"[ \t]+").ok();
    if let Some(re) = ws_re {
        text = re.replace_all(&text, " ").to_string();
    }
    let nl_re = regex::Regex::new(r"\n{3,}").ok();
    if let Some(re) = nl_re {
        text = re.replace_all(&text, "\n\n").to_string();
    }

    text = text.trim().to_string();

    if text.len() > max_chars {
        let end = text.floor_char_boundary(max_chars);
        text.truncate(end);
        if let Some(pos) = text.rfind('.')
            && pos > end / 2
        {
            text.truncate(pos + 1);
        }
    }

    (text, title)
}

fn strip_tags_single(html: &str) -> String {
    let re = regex::Regex::new(r"(?s)<[^>]+>").ok();
    match re {
        Some(r) => r.replace_all(html, " ").to_string(),
        None => html.to_string(),
    }
}
