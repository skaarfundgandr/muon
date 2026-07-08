use crate::application::bridge::BridgeChannels;
use crate::application::pipeline_runner::citation_verifier::{self, ValidCitation, VerificationOutput};
use crate::application::pipeline::PipelineStage;
use crate::application::services::report_builder;
use crate::config::MuonConfig;
use crate::domain::agents::clarifier::ClarifierResult;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::report::{ResearchReport, VerificationLevel};
use crate::domain::models::session::ReportStats;
use crate::domain::models::source::SourceType;
use crate::error::MuonError;
use crate::infrastructure::context::InfrastructureContext;
use crate::infrastructure::source_registry::SourceRegistry;

pub struct DeepResearcher<'a> {
    cfg: &'a MuonConfig,
    infra: &'a InfrastructureContext,
    bridge: &'a BridgeChannels,
}

impl<'a> DeepResearcher<'a> {
    pub fn new(
        cfg: &'a MuonConfig,
        infra: &'a InfrastructureContext,
        bridge: &'a BridgeChannels,
    ) -> Self {
        Self {
            cfg,
            infra,
            bridge,
        }
    }

    pub async fn run(
        &self,
        query: &str,
        plan: &ClarifierResult,
    ) -> Result<ResearchReport, MuonError> {
        let max_loops = self.cfg.agents.deep_researcher.iterations.max(1);
        let mut draft = String::new();
        let mut registry = SourceRegistry::new();
        if let Ok(sink) = self.infra.source_sink.lock() {
            for src in sink.sources() {
                registry.record(&src.url, src.source_type);
            }
        }
        let start = std::time::Instant::now();

        self.bridge
            .stage(PipelineStage::DeepResearch);
        self.bridge.log(
            AgentTag::Orchestrate,
            LogLevel::Info,
            format!("deep researcher started: max_loops={max_loops}"),
        );

        for loop_idx in 0..max_loops {
            let (planner_output, researcher_output) = futures::join!(
                self.planner_step(query, &draft, plan),
                self.researcher_step(query, &draft, plan, &mut registry),
            );
            let planner_output = planner_output?;
            let researcher_output = researcher_output?;

            draft = self
                .orchestrator_step(
                    query,
                    &draft,
                    plan,
                    &planner_output,
                    &researcher_output,
                )
                .await?;

            if loop_idx + 1 < max_loops {
                self.bridge.log(
                    AgentTag::Orchestrate,
                    LogLevel::Info,
                    format!("loop {} complete", loop_idx),
                );
            }
        }

        let unverified_report = self.to_report(&draft, &registry);

        // Embed sources into the vector store (non-fatal on error).
        if let Some(ref vs) = self.infra.vector_store {
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

    async fn planner_step(
        &self,
        query: &str,
        draft: &str,
        plan: &ClarifierResult,
    ) -> Result<String, MuonError> {
        let prompt = format!(
            "You are the Planner. Decompose the research query for the Orchestrator.\n\n\
             Query: {query}\nCurrent draft: {draft}\n\
             Clarifier plan: {}\n\n\
             Provide a focused outline with 3-5 sections and key points to research.",
            plan.plan_sections.join(", ")
        );
        let result = self.infra.planner.prompt_raw(&prompt).await?;
        self.bridge.log(
            AgentTag::Plan,
            LogLevel::Info,
            format!("planner produced plan (len={})", result.len()),
        );
        Ok(result)
    }

    async fn researcher_step(
        &self,
        query: &str,
        draft: &str,
        plan: &ClarifierResult,
        registry: &mut SourceRegistry,
    ) -> Result<String, MuonError> {
        let prior_context = if let Some(ref vs) = self.infra.vector_store {
            match vs.query(query, self.cfg.advanced.rag_top_k as usize).await {
                Ok(prior) if !prior.is_empty() => {
                    let items: Vec<String> = prior
                        .iter()
                        .map(|s| format!("- [{}] {}", s.title, s.snippet))
                        .collect();
                    format!("\nPrior knowledge:\n{}\n\n", items.join("\n"))
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };

        let prompt = format!(
            "You are the Researcher. Find concrete sources for the Orchestrator's draft.\n\
             Query: {query}\nDraft: {draft}\n\
             Focus: {}{prior_context}\n\
             Return a list of URLs and a one-line summary for each.",
            plan.plan_sections.join(", ")
        );
        let result = self.infra.researcher.prompt_raw(&prompt).await?;
        let urls = citation_verifier::extract_urls(&result)?;
        for url in &urls {
            registry.record(url.as_str(), SourceType::Web);
        }
        self.bridge.log(
            AgentTag::Search,
            LogLevel::Info,
            format!("researcher returned {} urls", urls.len()),
        );
        Ok(result)
    }

    async fn orchestrator_step(
        &self,
        query: &str,
        draft: &str,
        plan: &ClarifierResult,
        planner_output: &str,
        researcher_output: &str,
    ) -> Result<String, MuonError> {
        // Prompt content finalized in Phase 2 — placeholder kept minimal here only until 2.3.
        let prompt = format!(
            "You are the Orchestrator. Synthesize a complete markdown report.\n\n\
             Query: {query}\nPrevious draft: {draft}\n\n\
             Planner outline:\n{planner_output}\n\n\
             Researcher sources:\n{researcher_output}\n\n\
             Clarifier sections: {}\n\n\
             Write a comprehensive markdown report with sections under ## headings.",
            plan.plan_sections.join(", ")
        );
        let result = self.infra.deep_orchestrator.prompt_raw(&prompt).await?;
        self.bridge.log(
            AgentTag::Orchestrate,
            LogLevel::Info,
            format!("orchestrator produced draft (len={})", result.len()),
        );
        Ok(result)
    }
}
