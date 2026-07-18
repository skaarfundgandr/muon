use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::source::Source;
use crate::infrastructure::source_registry::SourceRegistry;

pub use crate::domain::models::report::VerificationLevel;
pub use normalize_url as normalize;

static URL_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?P<url>https?://[^\s\)\]\}\{\"\'\\<>]+)"#).ok());
static IPV4_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$").ok());
static NUMBERED_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\[(\d+)\]").ok());
static URL_REF_RE: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\[(https?://[^\]]+)\]").ok());

/// Output of the citation verification pipeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationOutput {
    pub verified_report: String,
    pub removed_citations: Vec<RemovedCitation>,
    pub valid_citations: Vec<ValidCitation>,
}

/// A citation that failed verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovedCitation {
    pub url: String,
    pub reason: RemovalReason,
}

/// Why a citation was removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemovalReason {
    UrlNotInRegistry,
    CitationKeyNotInRegistry,
    Unverifiable,
}

/// A citation that passed verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidCitation {
    pub url: String,
    pub level: VerificationLevel,
}

pub fn level_to_status(
    level: VerificationLevel,
) -> crate::domain::models::source::VerificationStatus {
    use crate::domain::models::source::VerificationStatus;
    match level {
        VerificationLevel::Exact => VerificationStatus::Exact,
        VerificationLevel::Prefix => VerificationStatus::Prefix,
        VerificationLevel::ChildPath => VerificationStatus::ChildPath,
        VerificationLevel::QuerySubset => VerificationStatus::QuerySubset,
    }
}

fn status_rank(s: crate::domain::models::source::VerificationStatus) -> u8 {
    use crate::domain::models::source::VerificationStatus;
    match s {
        VerificationStatus::Exact => 4,
        VerificationStatus::Prefix => 3,
        VerificationStatus::ChildPath => 2,
        VerificationStatus::QuerySubset => 1,
        VerificationStatus::Unverified | VerificationStatus::Removed => 0,
    }
}

pub fn apply_verification_to_sources(
    sources: &mut [crate::domain::models::source::Source],
    out: &VerificationOutput,
) {
    use crate::domain::models::source::VerificationStatus;

    for source in sources.iter_mut() {
        let mut status: Option<VerificationStatus> = None;
        for c in &out.valid_citations {
            if c.url.is_empty() {
                continue;
            }
            if match_url(&c.url, std::slice::from_ref(&source.url)).is_some() {
                let new = level_to_status(c.level);
                if status.is_none_or(|cur| status_rank(new) > status_rank(cur)) {
                    status = Some(new);
                }
            }
        }
        if status.is_none() {
            for c in &out.removed_citations {
                if c.url.is_empty() {
                    continue;
                }
                if match_url(&c.url, std::slice::from_ref(&source.url)).is_some() {
                    status = Some(VerificationStatus::Removed);
                    break;
                }
            }
        }
        if let Some(s) = status {
            source.verification_status = s;
            source.verified = !matches!(
                s,
                VerificationStatus::Unverified | VerificationStatus::Removed
            );
        }
    }
}

pub fn merge_verified_into_sink(
    sink: &mut SourceRegistry,
    sources: &mut [Source],
    out: &VerificationOutput,
) {
    apply_verification_to_sources(sources, out);
    for src in sources.iter() {
        if let Some(entry) = sink.sources_mut().iter_mut().find(|e| e.url == src.url) {
            entry.verification_status = src.verification_status;
            entry.verified = src.verified;
            if entry.relevance < src.relevance {
                entry.relevance = src.relevance;
            }
            if entry.title.is_empty() && !src.title.is_empty() {
                entry.title = src.title.clone();
            }
            if entry.snippet.is_empty() && !src.snippet.is_empty() {
                entry.snippet = src.snippet.clone();
            }
        } else {
            sink.record_source(src);
        }
    }
}

/// Verifies all citations in a report against a registry of known URLs and knowledge paths.
pub fn verify(
    report: &ResearchReport,
    registry_urls: &[String],
    knowledge_paths: &[String],
) -> Result<VerificationOutput, MuonError> {
    let mut valid = Vec::new();
    let mut removed = Vec::new();

    for cite in &report.citations {
        if !cite.url.is_empty() {
            let sanitized = sanitize(&cite.url)?;
            if !sanitized {
                removed.push(RemovedCitation {
                    url: cite.url.clone(),
                    reason: RemovalReason::Unverifiable,
                });
                continue;
            }
            if let Some(level) = match_url(&cite.url, registry_urls) {
                valid.push(ValidCitation {
                    url: cite.url.clone(),
                    level,
                });
            } else {
                removed.push(RemovedCitation {
                    url: cite.url.clone(),
                    reason: RemovalReason::UrlNotInRegistry,
                });
            }
        } else if is_knowledge_ref(&cite.context_snippet, knowledge_paths) {
            valid.push(ValidCitation {
                url: String::new(),
                level: VerificationLevel::Exact,
            });
        } else {
            removed.push(RemovedCitation {
                url: String::new(),
                reason: RemovalReason::CitationKeyNotInRegistry,
            });
        }
    }

    let verified_report = report_reflow(&report_to_markdown(report), &valid, &report.citations)?;

    Ok(VerificationOutput {
        verified_report,
        removed_citations: removed,
        valid_citations: valid,
    })
}

pub fn extract_urls(markdown: &str) -> Result<Vec<String>, MuonError> {
    let re = URL_RE.as_ref().ok_or_else(|| MuonError::Database("url regex failed".into()))?;
    let mut out = Vec::new();
    for cap in re.captures_iter(markdown) {
        let Some(m) = cap.name("url") else {
            continue;
        };
        let candidate = m
            .as_str()
            .trim_end_matches(['.', ',', ';', ':', ')', ']', '}', '"', '\'', '\\', '{', '>'])
            .to_string();
        if candidate.contains('"') || candidate.contains('\\') {
            continue;
        }
        let Ok(parsed) = url::Url::parse(&candidate) else {
            continue;
        };
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host().is_none() {
            continue;
        }
        out.push(candidate);
    }
    Ok(out)
}

/// Returns false if the URL is suspicious or unverifiable.
pub fn sanitize(url: &str) -> Result<bool, MuonError> {
    if url.contains("...") {
        return Ok(false);
    }

    if let Some(rest) = url.strip_prefix("http://") {
        let host = rest.split('/').next().unwrap_or("");
        let lc = host.to_lowercase();
        if lc == "bit.ly" || lc == "t.co" {
            return Ok(false);
        }
    } else if let Some(rest) = url.strip_prefix("https://") {
        let host = rest.split('/').next().unwrap_or("");
        let lc = host.to_lowercase();
        if lc == "bit.ly" || lc == "t.co" {
            return Ok(false);
        }
    }

    if let Some(rest) = url.strip_prefix("http://") {
        let host = rest.split('/').next().unwrap_or("");
        if is_ip_literal(host)? {
            return Ok(false);
        }
    } else if let Some(rest) = url.strip_prefix("https://") {
        let host = rest.split('/').next().unwrap_or("");
        if is_ip_literal(host)? {
            return Ok(false);
        }
    }

    let lower = url.to_lowercase();
    for scheme in &["javascript:", "data:", "vbscript:", "file:"] {
        if lower.starts_with(scheme) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Normalizes a URL for comparison: lowercase host, strip trailing slash, drop www., drop fragment.
pub fn normalize_url(url: &str) -> String {
    let lower = url.to_lowercase();
    let without_fragment = match lower.split_once('#') {
        Some((before, _)) => before.to_string(),
        None => lower,
    };
    let without_www = match without_fragment.strip_prefix("http://") {
        Some(rest) if rest.starts_with("www.") => {
            format!("http://{}", &rest[4..])
        }
        _ => match without_fragment.strip_prefix("https://") {
            Some(rest) if rest.starts_with("www.") => {
                format!("https://{}", &rest[4..])
            }
            _ => without_fragment,
        },
    };
    without_www.trim_end_matches('/').to_string()
}

fn is_ip_literal(host: &str) -> Result<bool, MuonError> {
    let h = host.trim_matches('[').trim_matches(']');
    let ipv4_re = IPV4_RE.as_ref().ok_or_else(|| MuonError::Database("ipv4 regex failed".into()))?;
    if ipv4_re.is_match(h) {
        return Ok(true);
    }
    if h.contains(':') {
        return Ok(true);
    }
    Ok(false)
}

/// Matches a URL against registry URLs using the 5-level pipeline.
pub fn match_url(url: &str, registry_urls: &[String]) -> Option<VerificationLevel> {
    let normalized = normalize_url(url);
    let norm_reg: Vec<String> = registry_urls.iter().map(|u| normalize_url(u)).collect();

    if norm_reg.iter().any(|r| r == &normalized) {
        return Some(VerificationLevel::Exact);
    }

    if let Some(area) = area(&normalized) {
        let count = norm_reg
            .iter()
            .filter(|r| {
                r.starts_with(&area) && r.get(area.len()..area.len() + 1).is_some_and(|c| c == "/")
            })
            .count();
        if count == 1 {
            return Some(VerificationLevel::Prefix);
        }
    }

    if norm_reg.iter().any(|r| normalized.starts_with(r)) {
        return Some(VerificationLevel::Prefix);
    }

    if child_path_match(&normalized, &norm_reg) {
        return Some(VerificationLevel::ChildPath);
    }

    if query_subset_match(&normalized, &norm_reg) {
        return Some(VerificationLevel::QuerySubset);
    }

    None
}

fn area(url: &str) -> Option<String> {
    let stripped = url.split('?').next()?;
    let stripped = stripped.split('#').next()?;
    let mut result = stripped.to_string();
    if let Some(idx) = result.rfind('/') {
        let after = &result[idx + 1..];
        if after.contains(|c: char| c.is_ascii_digit()) && after.len() > 8 {
            result.truncate(idx + 1);
        }
    }
    Some(result)
}

fn host_and_path(normalized: &str) -> Option<(&str, &str)> {
    let rest = normalized.split_once("://")?.1;
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    Some((host, path))
}

fn path_is_under(child: &str, parent: &str) -> bool {
    let parent = parent.trim_end_matches('/');
    let child = child.trim_end_matches('/');
    if parent.is_empty() {
        return false;
    }
    child == parent || child.starts_with(&format!("{parent}/"))
}

fn child_path_match(normalized: &str, registry: &[String]) -> bool {
    let Some((host, path)) = host_and_path(normalized) else {
        return false;
    };
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 2 {
        return false;
    }
    registry.iter().any(|r| {
        let Some((rh, rp)) = host_and_path(r) else {
            return false;
        };
        if rh != host {
            return false;
        }
        let r_segments: Vec<&str> = rp.split('/').filter(|s| !s.is_empty()).collect();
        if r_segments.len() < 2 {
            return false;
        }
        path_is_under(path, rp)
    })
}

fn query_subset_match(normalized: &str, registry: &[String]) -> bool {
    if let Some((base_url, query)) = normalized.split_once('?')
        && !query.is_empty()
    {
        let report_params = parse_query_params(query);
        for r in registry {
            if let Some((r_base, r_query)) = r.split_once('?')
                && base_url == r_base
                && !r_query.is_empty()
            {
                let reg_params = parse_query_params(r_query);
                if report_params.iter().all(|p| reg_params.contains(p)) {
                    return true;
                }
            }
        }
    }
    false
}

fn parse_query_params(query: &str) -> Vec<String> {
    query
        .split('&')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn is_knowledge_ref(text: &str, knowledge_paths: &[String]) -> bool {
    if knowledge_paths.is_empty() {
        return false;
    }
    knowledge_paths.iter().any(|kp| {
        text.to_lowercase().contains(&kp.to_lowercase())
            || kp.to_lowercase().contains(&text.to_lowercase())
    })
}

fn report_to_markdown(report: &ResearchReport) -> String {
    let mut parts = Vec::new();
    if !report.summary.is_empty() {
        parts.push(report.summary.clone());
    }
    for section in &report.sections {
        if !section.heading.is_empty() {
            parts.push(format!("## {}", section.heading));
        }
        if !section.body_markdown.is_empty() {
            parts.push(section.body_markdown.clone());
        }
    }
    parts.join("\n\n")
}

pub fn report_reflow(
    markdown: &str,
    valid_citations: &[ValidCitation],
    original_citations: &[crate::domain::models::report::Citation],
) -> Result<String, MuonError> {
    let mut url_to_new_num: HashMap<String, u32> = HashMap::new();
    for (i, cite) in valid_citations.iter().enumerate() {
        if !cite.url.is_empty() {
            url_to_new_num.insert(cite.url.clone(), (i + 1) as u32);
        }
    }

    let mut old_num_to_url: HashMap<u32, String> = HashMap::new();
    for c in original_citations {
        if !c.url.is_empty() {
            old_num_to_url.insert(c.reference_number, c.url.clone());
        }
    }

    let numbered_re = NUMBERED_RE.as_ref().ok_or_else(|| MuonError::Database("numbered ref regex failed".into()))?;
    let url_re = URL_REF_RE.as_ref().ok_or_else(|| MuonError::Database("url ref regex failed".into()))?;

    let mut old_to_new: HashMap<String, String> = HashMap::new();

    for cap in numbered_re.captures_iter(markdown) {
        if let Some(old) = cap.get(1) {
            let key = format!("[{}]", old.as_str());
            if let std::collections::hash_map::Entry::Vacant(e) = old_to_new.entry(key) {
                let n: u32 = old.as_str().parse().unwrap_or(0u32);
                let replacement = old_num_to_url
                    .get(&n)
                    .and_then(|url| url_to_new_num.get(url))
                    .map(|nn| format!("[{nn}]"))
                    .unwrap_or_default();
                e.insert(replacement);
            }
        }
    }

    for cap in url_re.captures_iter(markdown) {
        if let (Some(url_m), Some(full_m)) = (cap.get(1), cap.get(0)) {
            let url = url_m.as_str();
            if let Some(&n) = url_to_new_num.get(url) {
                old_to_new.insert(full_m.as_str().to_string(), format!("[{n}]"));
            } else {
                old_to_new.insert(full_m.as_str().to_string(), String::new());
            }
        }
    }

    let mut result = markdown.to_string();
    let mut placeholders: Vec<(String, String)> = Vec::new();
    for (i, (old, new)) in old_to_new.iter().enumerate() {
        let ph = format!("\u{E000}CITE{i}\u{E001}");
        result = result.replace(old, &ph);
        placeholders.push((ph, new.clone()));
    }
    for (ph, new) in placeholders {
        result = result.replace(&ph, &new);
    }
    Ok(result)
}
