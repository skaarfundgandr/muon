use crate::application::pipeline_runner::citation_verifier::VerificationOutput;
use crate::domain::agents::clarifier::ClarifierResult;
use crate::domain::models::report::{Citation, ReportSection, ResearchReport};
use crate::domain::models::session::ReportStats;
use crate::error::MuonError;

pub fn build(
    verified: VerificationOutput,
    plan: &ClarifierResult,
    elapsed_secs: u64,
) -> Result<ResearchReport, MuonError> {
    let (summary, sections) = split_sections(&verified.verified_report);

    let citations: Vec<Citation> = verified
        .valid_citations
        .iter()
        .enumerate()
        .map(|(i, vc)| Citation {
            reference_number: (i + 1) as u32,
            url: vc.url.clone(),
            title: derive_title(&vc.url),
            context_snippet: String::new(),
            verification_level: vc.level,
        })
        .collect();

    let total = verified.valid_citations.len() + verified.removed_citations.len();
    let stats = ReportStats {
        total_sources: total,
        verified_sources: verified.valid_citations.len(),
        removed_citations: verified.removed_citations.len(),
        elapsed_secs,
        tokens_in: 0,
        tokens_out: 0,
    };

    let title = plan
        .plan_title
        .clone()
        .unwrap_or_else(|| "Research Report".to_string());

    Ok(ResearchReport {
        title,
        summary,
        sections,
        citations,
        stats,
    })
}

pub fn split_sections(markdown: &str) -> (String, Vec<ReportSection>) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut sections = Vec::new();
    let mut summary_lines = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_body = Vec::new();
    let mut found_heading = false;

    for line in &lines {
        if let Some(heading) = line.strip_prefix("## ") {
            found_heading = true;
            if let Some(prev_heading) = current_heading.take() {
                let body = current_body.join("\n").trim().to_string();
                sections.push(ReportSection {
                    heading: prev_heading,
                    body_markdown: body,
                });
                current_body.clear();
            }
            current_heading = Some(heading.to_string());
        } else if found_heading {
            current_body.push(*line);
        } else {
            summary_lines.push(*line);
        }
    }

    if let Some(heading) = current_heading.take() {
        let body = current_body.join("\n").trim().to_string();
        sections.push(ReportSection {
            heading,
            body_markdown: body,
        });
    }

    let summary = summary_lines.join("\n").trim().to_string();
    (summary, sections)
}

pub fn derive_title(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    if let Ok(parsed) = url::Url::parse(url) {
        let path = parsed.path().trim_end_matches('/');
        if let Some(last) = path.rsplit('/').next()
            && !last.is_empty()
        {
            return last.replace(['-', '_'], " ");
        }
        let host = parsed.host_str().unwrap_or_default();
        if !host.is_empty() {
            return host.to_string();
        }
    }
    url.to_string()
}

