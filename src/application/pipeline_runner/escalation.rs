use crate::domain::models::report::ResearchReport;

const ESCALATION_KEYWORDS: &[&str] = &[
    "unable to find",
    "need more research",
    "i don't have enough information",
];

pub fn should_escalate(report: &ResearchReport) -> bool {
    if report.summary.trim().is_empty() {
        return true;
    }
    let tail: String = report
        .summary
        .chars()
        .rev()
        .take(800)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let lower = tail.to_lowercase();
    ESCALATION_KEYWORDS.iter().any(|k| lower.contains(k))
}
