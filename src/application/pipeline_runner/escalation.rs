use crate::domain::models::report::ResearchReport;

const ESCALATION_KEYWORDS: &[&str] = &[
    "unable to find",
    "need more research",
    "i don't have enough information",
];

pub fn report_response_text(report: &ResearchReport) -> String {
    let mut parts: Vec<String> = vec![report.summary.clone()];
    for section in &report.sections {
        parts.push(section.body_markdown.clone());
    }
    parts.join("\n")
}

pub fn should_escalate(report: &ResearchReport) -> bool {
    let full = report_response_text(report);
    if full.trim().is_empty() {
        return true;
    }
    let total = full.chars().count();
    let start = total.saturating_sub(800);
    let tail: String = full.chars().skip(start).collect();
    let lower = tail.to_lowercase();
    ESCALATION_KEYWORDS.iter().any(|k| lower.contains(k))
}
