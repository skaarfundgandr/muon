use std::future::Future;
use std::net::IpAddr;
use std::sync::LazyLock;

use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::domain::error::MuonError;

const NAME: &str = "fetch_page";
const MAX_BODY_BYTES: usize = 2_000_000;

static TITLE_RE: LazyLock<Option<regex::Regex>> =
    LazyLock::new(|| regex::Regex::new(r"(?is)<title[^>]*>(.*?)</title>").ok());
static TAG_RE: LazyLock<Option<regex::Regex>> =
    LazyLock::new(|| regex::Regex::new(r"(?s)<[^>]+>").ok());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
    Html,
    Pdf,
    Unsupported,
}

pub fn classify_body(content_type: Option<&str>, bytes: &[u8]) -> BodyKind {
    if bytes.starts_with(b"%PDF-") {
        return BodyKind::Pdf;
    }
    let ct = content_type
        .and_then(|v| v.split(';').next())
        .map(|v| v.trim().to_lowercase());
    match ct.as_deref() {
        Some("application/pdf") => BodyKind::Pdf,
        None | Some("") => BodyKind::Html,
        Some(t) if t.starts_with("text/") => BodyKind::Html,
        _ => BodyKind::Unsupported,
    }
}

pub fn html_bytes_to_output(bytes: &[u8], max_chars: usize) -> (String, Option<String>) {
    let html = String::from_utf8_lossy(bytes);
    let title = TITLE_RE.as_ref().and_then(|r| {
        r.captures(&html).and_then(|c| c.get(1)).map(|m| {
            let mut t = m.as_str().trim().to_string();
            if let Some(re) = TAG_RE.as_ref() {
                t = re.replace_all(&t, " ").to_string();
                t = t.trim().to_string();
            }
            t
        })
    });
    let md = html2md::rewrite_html(&html, false);
    let text = truncate_at_chars(&md, max_chars);
    (text, title)
}

pub fn pdf_bytes_to_text(bytes: &[u8], max_chars: usize) -> Result<String, MuonError> {
    let mut pdf = pdf_oxide::api::Pdf::from_bytes(bytes.to_vec()).map_err(|e| {
        MuonError::Search {
            provider: "fetch".into(),
            message: format!("pdf parse failed: {e}"),
        }
    })?;
    let page_count = pdf.page_count().map_err(|e| MuonError::Search {
        provider: "fetch".into(),
        message: format!("pdf page count failed: {e}"),
    })?;
    let mut out = String::new();
    for i in 0..page_count {
        let page_text = pdf.to_text(i).map_err(|e| MuonError::Search {
            provider: "fetch".into(),
            message: format!("pdf page {i} text extract failed: {e}"),
        })?;
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&page_text);
        if out.chars().count() >= max_chars {
            break;
        }
    }
    Ok(truncate_at_chars(&out, max_chars))
}

fn truncate_at_chars(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let end = text
        .char_indices()
        .nth(max_chars)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let mut s = text[..end].to_string();
    if let Some(pos) = s.rfind('.')
        && pos > s.len() / 2
    {
        s.truncate(pos + 1);
    }
    s
}

pub fn is_public_http_url(raw: &str) -> Result<(), String> {
    let url = Url::parse(raw).map_err(|e| format!("invalid url: {e}"))?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(format!("scheme not allowed: {other}")),
    }
    match url.host() {
        Some(url::Host::Domain(host)) => {
            if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
                return Err("localhost blocked".into());
            }
        }
        Some(url::Host::Ipv4(v4)) => {
            if is_blocked_ip(IpAddr::V4(v4)) {
                return Err("private or link-local address blocked".into());
            }
        }
        Some(url::Host::Ipv6(v6)) => {
            if is_blocked_ip(IpAddr::V6(v6)) {
                return Err("private or link-local address blocked".into());
            }
        }
        None => return Err("missing host".into()),
    }
    Ok(())
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
                || v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 64
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_ip(IpAddr::V4(v4));
            }
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

pub fn classify_and_render(
    content_type: Option<&str>,
    bytes: &[u8],
    truncated: bool,
    max_chars: usize,
) -> Result<(String, Option<String>), MuonError> {
    let kind = classify_body(content_type, bytes);
    if kind == BodyKind::Pdf && truncated {
        return Err(MuonError::Search {
            provider: "fetch".into(),
            message: format!(
                "PDF too large: response exceeds {} bytes; cannot parse truncated body",
                MAX_BODY_BYTES
            ),
        });
    }
    Ok(match kind {
        BodyKind::Html => html_bytes_to_output(bytes, max_chars),
        BodyKind::Pdf => {
            let text = pdf_bytes_to_text(bytes, max_chars)?;
            (text, None)
        }
        BodyKind::Unsupported => {
            return Err(MuonError::Search {
                provider: "fetch".into(),
                message: format!(
                    "unsupported content type: {}",
                    content_type.unwrap_or_default()
                ),
            });
        }
    })
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
        "Fetch a URL and return text. HTML becomes Markdown; PDFs are text-extracted; content is truncated. Unsupported content types are rejected.".to_string()
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
            if let Err(reason) = is_public_http_url(&args.url) {
                return Err(MuonError::Search {
                    provider: "fetch".into(),
                    message: format!("url blocked: {reason}"),
                });
            }

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

            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);

            let bytes = resp.bytes().await.map_err(|e| MuonError::Search {
                provider: "fetch".into(),
                message: format!("failed to read body from {}: {e}", args.url),
            })?;

            let truncated = bytes.len() > MAX_BODY_BYTES;
            let capped: &[u8] = if truncated {
                &bytes[..MAX_BODY_BYTES]
            } else {
                &bytes
            };

            let (text, title) =
                classify_and_render(content_type.as_deref(), capped, truncated, max_chars)?;

            Ok(FetchPageOutput {
                url: args.url,
                text,
                title,
            })
        }
    }
}
