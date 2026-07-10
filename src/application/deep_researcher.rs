use crate::application::bridge::BridgeChannels;
use crate::application::pipeline::PipelineStage;
use crate::application::pipeline_runner::citation_verifier::{
    self, ValidCitation, VerificationOutput,
};
use crate::application::services::report_builder;
use crate::config::MuonConfig;
use crate::domain::agents::clarifier::ClarifierResult;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::report::{ResearchReport, VerificationLevel};
use crate::domain::models::session::ReportStats;
use crate::domain::error::MuonError;
use crate::application::deps::PipelineDeps;
use crate::infrastructure::source_registry::SourceRegistry;

const GAVE_UP_PATTERNS: &[&str] = &[
    "please confirm",
    "do you want me to",
    "should i proceed",
    "i can't produce",
    "i cannot produce",
    "what i need from you",
];

pub struct DeepResearcher<'a> {
    cfg: &'a MuonConfig,
    deps: &'a PipelineDeps,
    bridge: &'a BridgeChannels,
}

impl<'a> DeepResearcher<'a> {
    pub fn new(
        cfg: &'a MuonConfig,
        deps: &'a PipelineDeps,
        bridge: &'a BridgeChannels,
    ) -> Self {
        Self { cfg, deps, bridge }
    }

    pub async fn run(
        &self,
        query: &str,
        plan: &ClarifierResult,
    ) -> Result<ResearchReport, MuonError> {
        let max_retries = self.cfg.agents.deep_researcher.max_retries.max(1);
        let mut draft = String::new();
        let mut registry = SourceRegistry::new();
        if let Ok(sink) = self.deps.source_sink.lock() {
            for src in sink.sources() {
                registry.record(&src.url, src.source_type);
            }
        }
        let start = std::time::Instant::now();

        self.bridge.stage(PipelineStage::DeepResearch);
        self.bridge.log(
            AgentTag::Orchestrate,
            LogLevel::Info,
            format!("deep researcher started: max_retries={max_retries}"),
        );

        let mut last_reason: Option<String> = None;

        for invoke_idx in 0..max_retries {
            let prompt = self.build_orchestrator_prompt(
                query,
                &draft,
                plan,
                invoke_idx,
                last_reason.as_deref(),
            );
            match self.deps.deep_orchestrator.prompt_raw(&prompt).await {
                Ok(text) => {
                    draft = text;
                    self.bridge.log(
                        AgentTag::Orchestrate,
                        LogLevel::Info,
                        format!("orchestrator produced draft (len={})", draft.len()),
                    );
                }
                Err(e) => {
                    if matches!(e, MuonError::MaxCycles { .. }) {
                        self.bridge.log(
                            AgentTag::Orchestrate,
                            LogLevel::Warn,
                            format!("orchestrator hit iteration limit (soft-fail): {e}"),
                        );
                        if draft.is_empty() {
                            draft = incomplete_orchestrator_stub(&registry);
                        }
                        break;
                    } else if !draft.is_empty() {
                        self.bridge.log(
                            AgentTag::Orchestrate,
                            LogLevel::Warn,
                            format!(
                                "orchestrator error with partial draft; producing partial report: {e}"
                            ),
                        );
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }

            let (complete, reason) = is_report_complete(
                &draft,
                &registry.urls(),
                self.cfg.agents.deep_researcher.min_report_length,
                self.cfg.agents.deep_researcher.min_report_sections,
            );
            if complete {
                self.bridge.log(
                    AgentTag::Orchestrate,
                    LogLevel::Info,
                    format!("loop {invoke_idx} complete: quality gate passed, exiting early"),
                );
                break;
            }

            last_reason = Some(reason);

            let remaining = invoke_idx + 1 < max_retries;
            let msg = if remaining {
                format!(
                    "loop {invoke_idx} complete: continuing (reason: {})",
                    last_reason.as_deref().unwrap_or("")
                )
            } else {
                format!(
                    "loop {invoke_idx} complete: quality gate not met, reached max iterations ({max_retries})"
                )
            };
            self.bridge.log(AgentTag::Orchestrate, LogLevel::Info, msg);
        }

        // Fold URLs discovered by tool calls during the loop (planner/researcher/orchestrator)
        // into the local registry so they appear in citations, embedding, and verification.
        if let Ok(sink) = self.deps.source_sink.lock() {
            for src in sink.sources() {
                registry.record(&src.url, src.source_type);
            }
        }

        let unverified_report = self.to_report(&draft, &registry);

        // Embed sources into the vector store (non-fatal on error).
        if let Some(ref vs) = self.deps.vector_store {
            for source in registry.sources_mut() {
                if source.embedding_id.is_none() && !source.snippet.is_empty() {
                    match vs.add(source, &source.snippet).await {
                        Ok(Some(id)) => source.embedding_id = Some(id),
                        Ok(None) => {}
                        Err(e) => {
                            self.bridge.log(
                                AgentTag::Sys,
                                LogLevel::Warn,
                                format!("embed failed for {}: {e}", source.url),
                            );
                        }
                    }
                }
            }
        }

        let registry_urls = registry.urls();
        self.bridge.stage(PipelineStage::CitationVerify);
        let verified = if self.cfg.agents.deep_researcher.citation_verify {
            citation_verifier::verify(&unverified_report, &registry_urls, &[])?
        } else {
            VerificationOutput {
                verified_report: draft.clone(),
                removed_citations: Vec::new(),
                valid_citations: registry
                    .sources()
                    .iter()
                    .map(|s| ValidCitation {
                        url: s.url.clone(),
                        level: VerificationLevel::Exact,
                    })
                    .collect(),
            }
        };
        let elapsed = start.elapsed().as_secs();
        self.bridge.stage(PipelineStage::Report);
        let final_report = report_builder::build(verified, plan, elapsed)?;
        self.bridge.log(
            AgentTag::Verify,
            LogLevel::Info,
            format!(
                "citation verification complete: {} verified, {} removed",
                final_report.stats.verified_sources, final_report.stats.removed_citations
            ),
        );
        Ok(final_report)
    }

    fn build_orchestrator_prompt(
        &self,
        query: &str,
        draft: &str,
        plan: &ClarifierResult,
        invoke_idx: u64,
        last_reason: Option<&str>,
    ) -> String {
        let mut prompt = format!(
            "You are the Orchestrator for a deep-research pipeline. USE your tools: \
             call `delegate_planner` to get a research plan, then `delegate_researcher` \
             (possibly multiple times) to gather findings, and `think` to reason. \
             You also have `web_search` and `paper_search` if configured.\n\n\
             Query: {query}\nPrevious draft (refine, don't discard):\n{draft}\n\n\
             Clarifier sections to cover: {}\n\n\
             Requirements:\n  \
             - Begin with a `# Title` line, then a 2-3 sentence summary.\n  \
             - Body sections under `## Section Title` headings.\n  \
             - Cite inline as markdown links to source URLs from the Researcher or your web_search.\n  \
             - Do not invent sources.\n",
            plan.plan_sections.join(", ")
        );
        if invoke_idx > 0
            && let Some(reason) = last_reason
        {
            prompt.push_str(&format!(
                "\nYour previous report was incomplete. Reason: {reason}. \
                 Produce a complete, self-contained markdown report now.\n"
            ));
        }
        prompt
    }

    fn to_report(&self, draft: &str, registry: &SourceRegistry) -> ResearchReport {
        let lines: Vec<&str> = draft.lines().collect();
        let mut title = String::new();
        let mut summary_lines = Vec::new();
        let mut sections = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_body = Vec::new();
        let mut found_heading = false;

        for line in &lines {
            if let Some(h) = line.strip_prefix("# ") {
                if title.is_empty() {
                    title = h.trim().to_string();
                }
                if !found_heading {
                    summary_lines.push(*line);
                } else {
                    current_body.push(*line);
                }
            } else if let Some(heading) = line.strip_prefix("## ") {
                found_heading = true;
                if let Some(prev) = current_heading.take() {
                    let body = current_body.join("\n").trim().to_string();
                    sections.push(crate::domain::models::report::ReportSection {
                        heading: prev,
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
            sections.push(crate::domain::models::report::ReportSection {
                heading,
                body_markdown: body,
            });
        }

        if title.is_empty() {
            title = "Research Report".to_string();
        }

        let summary = summary_lines.join("\n").trim().to_string();
        let citations = registry
            .sources()
            .iter()
            .enumerate()
            .map(|(i, s)| crate::domain::models::report::Citation {
                reference_number: (i + 1) as u32,
                url: s.url.clone(),
                title: s.title.clone(),
                context_snippet: s.snippet.clone(),
                verification_level: VerificationLevel::Exact,
            })
            .collect();

        let total_sources = registry.sources().len();
        ResearchReport {
            title,
            summary,
            sections,
            citations,
            stats: ReportStats {
                total_sources,
                ..ReportStats::default()
            },
        }
    }
}

fn incomplete_orchestrator_stub(registry: &SourceRegistry) -> String {
    let mut out = String::from(
        "# Research Report\n\n\
         The research orchestrator reached its iteration limit before producing \
         a complete draft. Findings may be incomplete.\n",
    );
    let sources = registry.sources();
    if !sources.is_empty() {
        out.push_str("\n## Sources discovered before limit\n\n");
        for (i, s) in sources.iter().enumerate() {
            let title = if s.title.is_empty() {
                s.url.as_str()
            } else {
                s.title.as_str()
            };
            out.push_str(&format!("{}. [{}]({})\n", i + 1, title, s.url));
            if !s.snippet.is_empty() {
                out.push_str(&format!("   {}\n", s.snippet));
            }
        }
    }
    out
}

pub fn is_report_complete(
    draft: &str,
    registry_urls: &[String],
    min_length: u64,
    min_sections: u64,
) -> (bool, String) {
    let len_chars = draft.chars().count() as u64;
    if len_chars < min_length {
        return (
            false,
            format!("draft too short ({len_chars} chars, need {min_length})"),
        );
    }

    let section_count = draft.lines().filter(|l| l.starts_with("## ")).count() as u64;
    if section_count < min_sections {
        return (
            false,
            format!("missing section headers ({section_count} found, need {min_sections})"),
        );
    }

    let has_sources_section = draft.lines().any(|l| {
        l.strip_prefix("## ")
            .map(|h| {
                let h = h.to_lowercase();
                h.contains("source") || h.contains("reference")
            })
            .unwrap_or(false)
    });
    let cites_known =
        !registry_urls.is_empty() && registry_urls.iter().any(|u| draft.contains(u.as_str()));
    if !has_sources_section && !registry_urls.is_empty() && !cites_known {
        return (false, "no valid citations / sources section".to_string());
    }

    let lower = draft.to_lowercase();
    if let Some(matched) = GAVE_UP_PATTERNS.iter().find(|p| lower.contains(*p)) {
        return (false, format!("agent gave up signal: \"{matched}\""));
    }

    (true, String::new())
}
