use muon::domain::models::log_entry::AgentTag;
use muon::domain::models::log_entry::LogEntry;
use muon::domain::models::log_entry::LogLevel;
use muon::domain::models::query::{Depth, Intent, QueryIntent};
use muon::domain::models::report::Citation;
use muon::domain::models::report::ReportSection;
use muon::domain::models::report::ResearchReport;
use muon::domain::models::report::VerificationLevel;
use muon::domain::models::research_plan::ResearchPlan;
use muon::domain::models::session::ReportStats;
use muon::domain::models::session::Session;
use muon::domain::models::session::SessionStatus;
use muon::domain::models::source::Source;
use muon::domain::models::source::SourceType;
use muon::domain::models::source::VerificationStatus;
use muon::application::pipeline::PipelineStage;
use muon::domain::agents::intent_classifier::parse_intent;
use uuid::Uuid;

// ── intent_classifier ─────────────────────────────────────────────────

#[test]
fn parse_research_shallow() -> Result<(), Box<dyn std::error::Error>> {
    let q = parse_intent(r#"{"intent":"research","depth":"shallow"}"#)?;
    assert!(matches!(q.intent, Intent::Research));
    assert_eq!(q.depth, Depth::Shallow);
    Ok(())
}

#[test]
fn parse_research_deep() -> Result<(), Box<dyn std::error::Error>> {
    let q = parse_intent(r#"{"intent":"research","depth":"deep"}"#)?;
    assert!(matches!(q.intent, Intent::Research));
    assert_eq!(q.depth, Depth::Deep);
    Ok(())
}

#[test]
fn parse_meta_response() -> Result<(), Box<dyn std::error::Error>> {
    let q = parse_intent(r#"{"intent":"meta","response":"hi"}"#)?;
    match q.intent {
        Intent::Meta(s) => assert_eq!(s, "hi"),
        Intent::Research => return Err("expected meta, got research".into()),
    }
    Ok(())
}

#[test]
fn parse_defaults_depth_to_shallow() -> Result<(), Box<dyn std::error::Error>> {
    let q = parse_intent(r#"{"intent":"research"}"#)?;
    assert_eq!(q.depth, Depth::Shallow);
    Ok(())
}

#[test]
fn parse_rejects_non_json() {
    assert!(parse_intent("not json").is_err());
}

#[test]
fn parse_rejects_missing_intent() {
    assert!(parse_intent(r#"{"depth":"shallow"}"#).is_err());
}

// ── models: serde round-trips ──────────────────────────────────────────

#[test]
fn session_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let session = Session {
        id: Uuid::new_v4(),
        query: "what is rust".to_string(),
        status: SessionStatus::Researching,
        pipeline_stage: PipelineStage::DeepResearch,
        intent: None,
        plan: None,
        clarifier_result: None,
        sources: Vec::new(),
        report: None,
        logs: Vec::new(),
        stats: ReportStats::default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&session)?;
    let back: Session = serde_json::from_str(&json)?;
    assert_eq!(back.id, session.id);
    assert_eq!(back.query, session.query);
    assert_eq!(back.status, SessionStatus::Researching);
    assert_eq!(back.pipeline_stage, PipelineStage::DeepResearch);
    Ok(())
}

#[test]
fn citation_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let c = Citation {
        reference_number: 1,
        url: "https://example.com".to_string(),
        title: "Example".to_string(),
        context_snippet: "snippet".to_string(),
        verification_level: VerificationLevel::Exact,
    };
    let json = serde_json::to_string(&c)?;
    let back: Citation = serde_json::from_str(&json)?;
    assert_eq!(back.reference_number, 1);
    assert_eq!(back.verification_level, VerificationLevel::Exact);
    Ok(())
}

#[test]
fn direct_report_has_default_title() {
    let r = ResearchReport::direct("answer text");
    assert_eq!(r.title, "Direct Answer");
    assert_eq!(r.summary, "answer text");
    assert!(r.sections.is_empty());
    assert!(r.citations.is_empty());
}

#[test]
fn source_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let s = Source {
        url: "https://example.com".to_string(),
        title: "t".to_string(),
        snippet: "s".to_string(),
        relevance: 0.95,
        source_type: SourceType::Web,
        verified: true,
        verification_status: VerificationStatus::Exact,
        embedding_id: Some("emb-1".to_string()),
    };
    let json = serde_json::to_string(&s)?;
    let back: Source = serde_json::from_str(&json)?;
    assert_eq!(back.url, s.url);
    assert_eq!(back.source_type, SourceType::Web);
    assert_eq!(back.verification_status, VerificationStatus::Exact);
    Ok(())
}

#[test]
fn query_intent_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let q = QueryIntent {
        intent: Intent::Research,
        depth: Depth::Deep,
    };
    let json = serde_json::to_string(&q)?;
    let back: QueryIntent = serde_json::from_str(&json)?;
    assert_eq!(q.depth, back.depth);
    Ok(())
}

#[test]
fn research_plan_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let p = ResearchPlan {
        title: "Plan".to_string(),
        sections: vec!["intro".to_string(), "body".to_string()],
        approved: true,
    };
    let json = serde_json::to_string(&p)?;
    let back: ResearchPlan = serde_json::from_str(&json)?;
    assert_eq!(back.title, "Plan");
    assert_eq!(back.sections.len(), 2);
    assert!(back.approved);
    Ok(())
}

#[test]
fn log_entry_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let e = LogEntry {
        timestamp: chrono::Utc::now(),
        agent_tag: AgentTag::Search,
        message: "hello".to_string(),
        level: LogLevel::Info,
    };
    let json = serde_json::to_string(&e)?;
    let back: LogEntry = serde_json::from_str(&json)?;
    assert_eq!(back.agent_tag, AgentTag::Search);
    assert_eq!(back.level, LogLevel::Info);
    Ok(())
}

#[test]
fn report_section_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let s = ReportSection {
        heading: "h".to_string(),
        body_markdown: "body".to_string(),
    };
    let json = serde_json::to_string(&s)?;
    let back: ReportSection = serde_json::from_str(&json)?;
    assert_eq!(back.heading, "h");
    assert_eq!(back.body_markdown, "body");
    Ok(())
}
