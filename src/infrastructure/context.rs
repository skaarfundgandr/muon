use crate::domain::error::MuonError;
use crate::domain::models::log_entry::AgentTag;
use crate::domain::traits::agent::MuonAgent;
use crate::domain::traits::session_store::SessionStore;
use crate::infrastructure::source_registry::SourceRegistry;
use std::sync::{Arc, Mutex};

pub struct InfrastructureContext {
    pub(crate) intent_classifier: Arc<dyn MuonAgent>,
    pub(crate) shallow: Arc<dyn MuonAgent>,
    pub(crate) clarifier: Arc<dyn MuonAgent>,
    pub(crate) deep_orchestrator: Arc<dyn MuonAgent>,
    pub(crate) planner: Arc<dyn MuonAgent>,
    pub(crate) researcher: Arc<dyn MuonAgent>,
    pub(crate) session_store: Arc<dyn SessionStore>,
    pub(crate) source_sink: Arc<Mutex<SourceRegistry>>,
    pub(crate) vector_store: Option<Arc<dyn crate::domain::traits::vector_store::VectorStore>>,
}

impl std::fmt::Debug for InfrastructureContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfrastructureContext")
            .finish_non_exhaustive()
    }
}

impl InfrastructureContext {
    pub fn new(
        intent_classifier: Arc<dyn MuonAgent>,
        shallow: Arc<dyn MuonAgent>,
        clarifier: Arc<dyn MuonAgent>,
        deep_orchestrator: Arc<dyn MuonAgent>,
        planner: Arc<dyn MuonAgent>,
        researcher: Arc<dyn MuonAgent>,
        session_store: Arc<dyn SessionStore>,
    ) -> Self {
        Self::with_sink(
            intent_classifier,
            shallow,
            clarifier,
            deep_orchestrator,
            planner,
            researcher,
            session_store,
            Arc::new(Mutex::new(SourceRegistry::new())),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_sink(
        intent_classifier: Arc<dyn MuonAgent>,
        shallow: Arc<dyn MuonAgent>,
        clarifier: Arc<dyn MuonAgent>,
        deep_orchestrator: Arc<dyn MuonAgent>,
        planner: Arc<dyn MuonAgent>,
        researcher: Arc<dyn MuonAgent>,
        session_store: Arc<dyn SessionStore>,
        source_sink: Arc<Mutex<SourceRegistry>>,
        vector_store: Option<Arc<dyn crate::domain::traits::vector_store::VectorStore>>,
    ) -> Self {
        Self {
            intent_classifier,
            shallow,
            clarifier,
            deep_orchestrator,
            planner,
            researcher,
            session_store,
            source_sink,
            vector_store,
        }
    }

    pub async fn new_live(
        cfg: &crate::application::config::MuonConfig,
        bridge: &crate::application::bridge::BridgeChannels,
    ) -> Result<Self, MuonError> {
        use crate::infrastructure::providers::ProviderClient;
        use rig_core::client::CompletionClient;
        use std::sync::Arc;

        let factory = crate::infrastructure::agent_rs::ReActFactory::new(cfg, bridge.clone());
        let preamble = &cfg.advanced.agent_preamble;
        let source_sink = Arc::new(Mutex::new(SourceRegistry::new()));

        let providers = &cfg.providers;
        if providers.is_empty() {
            bridge.log(
                crate::domain::models::log_entry::AgentTag::Sys,
                crate::domain::models::log_entry::LogLevel::Warn,
                "no [[providers]] configured — starting with degraded stub agents",
            );
            let pool =
                crate::infrastructure::storage::init_pool(&cfg.advanced.session_db_path).await?;
            let session_store: Arc<dyn SessionStore> = Arc::new(
                crate::infrastructure::storage::DieselSessionStore::new(pool),
            );
            return Ok(Self::with_sink(
                Arc::new(
                    crate::infrastructure::agent_stubs::ConfigRequiredAgent::new(AgentTag::Intent),
                ),
                Arc::new(
                    crate::infrastructure::agent_stubs::ConfigRequiredAgent::new(AgentTag::Search),
                ),
                Arc::new(
                    crate::infrastructure::agent_stubs::ConfigRequiredAgent::new(AgentTag::Clarify),
                ),
                Arc::new(
                    crate::infrastructure::agent_stubs::ConfigRequiredAgent::new(
                        AgentTag::Orchestrate,
                    ),
                ),
                Arc::new(
                    crate::infrastructure::agent_stubs::ConfigRequiredAgent::new(AgentTag::Plan),
                ),
                Arc::new(
                    crate::infrastructure::agent_stubs::ConfigRequiredAgent::new(AgentTag::Search),
                ),
                session_store,
                source_sink,
                None,
            ));
        }
        fn resolve_model_id(
            providers: &[crate::application::config::ProviderConfig],
            provider_name: &str,
            model_name: &str,
        ) -> String {
            providers
                .iter()
                .find(|p| p.name == provider_name)
                .and_then(|p| p.models.iter().find(|m| m.name == model_name))
                .and_then(|m| {
                    if m.model_id.is_empty() {
                        None
                    } else {
                        Some(m.model_id.clone())
                    }
                })
                .unwrap_or_else(|| model_name.to_string())
        }
        let resolve_client = |name: &str| {
            if name.is_empty() {
                crate::infrastructure::providers::ResolvedClient::for_default_provider(providers)
            } else {
                crate::infrastructure::providers::ResolvedClient::for_named_provider(
                    name, providers,
                )
            }
        };


        macro_rules! build_agent {
            ($client:expr, $model:expr, |$c:ident| $body:expr) => {
                match $client {
                    ProviderClient::OpenAI($c) | ProviderClient::OpenAICompatible($c) => $body,
                    ProviderClient::Gemini($c) => $body,
                    ProviderClient::Anthropic($c) => $body,
                }
            };
        }

        let web_provider: Option<Arc<dyn crate::domain::traits::search_provider::SearchProvider>> =
            crate::infrastructure::search::provider::resolve_web_provider(cfg);
        let paper_providers: Vec<Arc<dyn crate::domain::traits::search_provider::SearchProvider>> =
            crate::infrastructure::search::provider::resolve_paper_providers(cfg);
        let fetch_http = reqwest::Client::new();

        let user_agents_dir: std::path::PathBuf =
            crate::infrastructure::util::expand_tilde(cfg.advanced.agents_dir.clone());
        let repo_agents_dir: std::path::PathBuf =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agents");

        let load_agent_def =
            |name: &str, fallback_preamble: &str| -> Result<crate::application::config::AgentDef, MuonError> {
                for dir in [user_agents_dir.as_path(), repo_agents_dir.as_path()] {
                    match crate::infrastructure::config::load_by_name(dir, name) {
                        Ok(Some(mut def)) => {
                            if def.preamble_markdown.is_empty() {
                                def.preamble_markdown = fallback_preamble.to_string();
                            }
                            return Ok(def);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            return Err(MuonError::Config(format!(
                                "agent definition '{name}' in {dir:?}: {e}"
                            )));
                        }
                    }
                }
                Err(MuonError::Config(format!(
                    "agent definition '{name}' not found in {} or {}",
                    user_agents_dir.display(),
                    repo_agents_dir.display()
                )))
            };

        let orchestrator_def = load_agent_def("deep-orchestrator", preamble)?;
        let planner_def = load_agent_def("planner", preamble)?;
        let researcher_def = load_agent_def("researcher", preamble)?;
        let clarifier_def = load_agent_def(
            "clarifier",
            "You are a clarifying and planning agent. Ask clarifying questions to resolve ambiguities in the user's query, and construct a detailed research plan. You must respond with strict JSON only.",
        )?;
        let intent_def = load_agent_def(
            "intent-classifier",
            "You are an intent classifier. Classify the user's query and respond with STRICT JSON only \u{2014} no other text, no markdown, no explanation:\n{\"intent\": \"research\"|\"meta\", \"depth\": \"shallow\"|\"deep\"}\nIf intent is \"meta\", also include a \"response\" field with a direct answer.",
        )?;
        let shallow_def = load_agent_def("shallow-researcher", preamble)?;

        let intent_client = resolve_client(&intent_def.provider)?;
        let shallow_client = resolve_client(&shallow_def.provider)?;
        let clarifier_client = resolve_client(&clarifier_def.provider)?;
        let deep_client = resolve_client(&orchestrator_def.provider)?;
        let planner_client = resolve_client(&planner_def.provider)?;
        let researcher_client = resolve_client(&researcher_def.provider)?;

        let orchestrator_preamble = &orchestrator_def.preamble_markdown;
        let planner_preamble = &planner_def.preamble_markdown;
        let researcher_preamble = &researcher_def.preamble_markdown;
        let clarifier_preamble = &clarifier_def.preamble_markdown;
        let intent_preamble = &intent_def.preamble_markdown;
        let shallow_preamble = &shallow_def.preamble_markdown;

        // Intent Classifier — no tools.
        let intent_classifier: Arc<dyn MuonAgent> =
            Arc::new(crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Intent,
                build_agent!(
                    &intent_client.client,
                    &resolve_model_id(
                        providers,
                        &intent_def.provider,
                        &intent_def.model
                    ),
                    |c| {
                        let agent = c
                            .agent(resolve_model_id(
                                providers,
                                &intent_def.provider,
                                &intent_def.model,
                            ))
                            .preamble(intent_preamble)
                            .temperature(intent_def.temperature)
                            .max_tokens(u64::from(intent_def.max_tokens))
                            .hook(agent_rs::observability::LangSmithAgentHook)
                            .build();
                        factory.build_runner(
                            agent,
                            AgentTag::Intent,
                            5,
                            intent_def.timeout_secs,
                            source_sink.clone(),
                            Some(crate::infrastructure::agent_rs::react_agents::REMINDER_FINALIZE),
                            true,
                        )
                    }
                ),
                bridge.clone(),
            ));

        // Shallow Researcher — fetch_page (always), web_search (if configured).
        let shallow: Arc<dyn MuonAgent> =
            Arc::new(crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Search,
                build_agent!(
                    &shallow_client.client,
                    &resolve_model_id(
                        providers,
                        &shallow_def.provider,
                        &shallow_def.model
                    ),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(
                                providers,
                                &shallow_def.provider,
                                &shallow_def.model,
                            ))
                            .preamble(shallow_preamble)
                            .temperature(shallow_def.temperature)
                            .max_tokens(u64::from(shallow_def.max_tokens))
                            .tool(crate::infrastructure::agent_rs::tools::FetchPageTool::new(
                                fetch_http.clone(),
                                8000,
                            ));
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(
                                wp.clone(),
                            ))
                        } else {
                            b
                        };
                        let max_turns =
                            cfg.agents.shallow_researcher.max_tool_iters.max(1) as usize;
                        let max_cycles =
                            cfg.agents.shallow_researcher.max_llm_turns.max(1) as usize;
                        let shallow_hook = crate::infrastructure::agent_rs::researcher_hook(
                            bridge.clone(),
                            AgentTag::Search,
                            source_sink.clone(),
                        );
                        let agent = b.default_max_turns(max_turns).hook(shallow_hook).build();
                        factory.build_runner(
                            agent,
                            AgentTag::Search,
                            max_cycles,
                            shallow_def.timeout_secs,
                            source_sink.clone(),
                            Some(crate::infrastructure::agent_rs::react_agents::REMINDER_FINALIZE),
                            false,
                        )
                    }
                ),
                bridge.clone(),
            ));

        // Clarifier — web_search (if configured).
        let compaction_threshold = (cfg.advanced.compaction_threshold * 100.0) as usize;
        let clarifier: Arc<dyn MuonAgent> =
            Arc::new(crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Clarify,
                build_agent!(
                    &clarifier_client.client,
                    &resolve_model_id(
                        providers,
                        &clarifier_def.provider,
                        &clarifier_def.model
                    ),
                    |c| {
                        let agent = if let Some(ref wp) = web_provider {
                            c.agent(resolve_model_id(
                                providers,
                                &clarifier_def.provider,
                                &clarifier_def.model,
                            ))
                            .preamble(clarifier_preamble)
                            .temperature(clarifier_def.temperature)
                            .max_tokens(u64::from(clarifier_def.max_tokens))
                            .tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(
                                wp.clone(),
                            ))
                            .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                            .hook(agent_rs::observability::LangSmithAgentHook)
                            .build()
                        } else {
                            c.agent(resolve_model_id(
                                providers,
                                &clarifier_def.provider,
                                &clarifier_def.model,
                            ))
                            .preamble(clarifier_preamble)
                            .temperature(clarifier_def.temperature)
                            .max_tokens(u64::from(clarifier_def.max_tokens))
                            .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                            .hook(agent_rs::observability::LangSmithAgentHook)
                            .build()
                        };
                        factory.build_clarifier_runner(
                            agent,
                            AgentTag::Clarify,
                            cfg.agents.clarifier.max_iterations as usize,
                            clarifier_def.timeout_secs,
                            compaction_threshold,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ));

        // Planner — think (always), web_search + paper_search (if configured).
        let planner: Arc<dyn MuonAgent> =
            Arc::new(crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Plan,
                build_agent!(
                    &planner_client.client,
                    &resolve_model_id(
                        providers,
                        &planner_def.provider,
                        &planner_def.model
                    ),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(
                                providers,
                                &planner_def.provider,
                                &planner_def.model,
                            ))
                            .preamble(planner_preamble)
                            .temperature(planner_def.temperature)
                            .max_tokens(u64::from(planner_def.max_tokens))
                            .tool(crate::infrastructure::agent_rs::tools::ThinkTool);
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(
                                wp.clone(),
                            ))
                        } else {
                            b
                        };
                        let b = if !paper_providers.is_empty() {
                            b.tool(
                                crate::infrastructure::agent_rs::tools::PaperSearchTool::new(
                                    paper_providers.clone(),
                                ),
                            )
                        } else {
                            b
                        };
                        let agent = b
                            .default_max_turns(
                                cfg.agents.deep_researcher.planner_max_tool_calls.max(1) as usize,
                            )
                            .hook(agent_rs::observability::LangSmithAgentHook)
                            .build();
                        factory.build_planner_runner(
                            agent,
                            AgentTag::Plan,
                            cfg.agents.deep_researcher.planner_max_cycles.max(1) as usize,
                            planner_def.timeout_secs,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ));

        // Researcher — managed multi-turn (non-ReAct): search/fetch then answer.
        let researcher_max_turns =
            cfg.agents.deep_researcher.researcher_max_tool_calls.max(1) as usize;
        let researcher_hook = crate::infrastructure::agent_rs::researcher_hook(
            bridge.clone(),
            AgentTag::Search,
            source_sink.clone(),
        );
        let researcher: Arc<dyn MuonAgent> = Arc::new(build_agent!(
            &researcher_client.client,
            &resolve_model_id(
                providers,
                &researcher_def.provider,
                &researcher_def.model
            ),
            |c| {
                let b = c
                    .agent(resolve_model_id(
                        providers,
                        &researcher_def.provider,
                        &researcher_def.model,
                    ))
                    .preamble(researcher_preamble)
                    .temperature(researcher_def.temperature)
                    .max_tokens(u64::from(researcher_def.max_tokens))
                    .tool(crate::infrastructure::agent_rs::tools::FetchPageTool::new(
                        fetch_http.clone(),
                        8000,
                    ));
                let b = if let Some(ref wp) = web_provider {
                    b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(
                        wp.clone(),
                    ))
                } else {
                    b
                };
                let b = if !paper_providers.is_empty() {
                    b.tool(
                        crate::infrastructure::agent_rs::tools::PaperSearchTool::new(
                            paper_providers.clone(),
                        ),
                    )
                } else {
                    b
                };
                let agent = b
                    .default_max_turns(researcher_max_turns)
                    .hook(researcher_hook.clone())
                    .build();
                crate::infrastructure::agent_rs::ManagedAgent::from_rig_agent_with_hook(
                    AgentTag::Search,
                    agent,
                    bridge.clone(),
                )
            }
        ));

        // Deep Orchestrator — think (always), web_search + paper_search (if configured).
        let deep_orchestrator: Arc<dyn MuonAgent> = Arc::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Orchestrate,
                build_agent!(
                    &deep_client.client,
                    &resolve_model_id(
                        providers,
                        &orchestrator_def.provider,
                        &orchestrator_def.model
                    ),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(
                                providers,
                                &orchestrator_def.provider,
                                &orchestrator_def.model,
                            ))
                            .preamble(orchestrator_preamble)
                            .temperature(orchestrator_def.temperature)
                            .max_tokens(u64::from(orchestrator_def.max_tokens))
                            .tool(crate::infrastructure::agent_rs::tools::ThinkTool)
                            .tool(crate::infrastructure::agent_rs::SubagentTool::<
                                crate::infrastructure::agent_rs::PlannerKind,
                            >::new(std::sync::Arc::clone(
                                &planner,
                            )))
                            .tool(crate::infrastructure::agent_rs::SubagentTool::<
                                crate::infrastructure::agent_rs::ResearcherKind,
                            >::new(std::sync::Arc::clone(
                                &researcher,
                            )));
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(
                                wp.clone(),
                            ))
                        } else {
                            b
                        };
                        let b = if !paper_providers.is_empty() {
                            b.tool(
                                crate::infrastructure::agent_rs::tools::PaperSearchTool::new(
                                    paper_providers.clone(),
                                ),
                            )
                        } else {
                            b
                        };
                        let agent = b
                            .default_max_turns(
                                cfg.agents
                                    .deep_researcher
                                    .orchestrator_max_tool_calls
                                    .max(1) as usize,
                            )
                            .hook(agent_rs::observability::LangSmithAgentHook)
                            .build();
                        factory.build_runner(
                            agent,
                            AgentTag::Orchestrate,
                            cfg.agents.deep_researcher.iterations.max(1) as usize,
                            orchestrator_def.timeout_secs,
                            source_sink.clone(),
                            Some(
                                crate::infrastructure::agent_rs::react_agents::REMINDER_ORCHESTRATOR,
                            ),
                            true,
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        let pool = crate::infrastructure::storage::init_pool(&cfg.advanced.session_db_path).await?;
        let session_store: Arc<dyn SessionStore> = Arc::new(
            crate::infrastructure::storage::DieselSessionStore::new(pool),
        );

        // RAG — optional, fails open (no RAG) if the model/index can't init.
        let vector_store: Option<Arc<dyn crate::domain::traits::vector_store::VectorStore>> =
            match crate::infrastructure::rag::RagContext::open(cfg).await {
                Ok(rag) => {
                    use crate::domain::models::log_entry::{AgentTag, LogLevel};
                    bridge.log(AgentTag::Sys, LogLevel::Info, "RAG context initialized");
                    Some(Arc::new(rag))
                }
                Err(e) => {
                    use crate::domain::models::log_entry::{AgentTag, LogLevel};
                    bridge.log(
                        AgentTag::Sys,
                        LogLevel::Warn,
                        format!("RAG init failed, continuing without: {e}"),
                    );
                    None
                }
            };

        Ok(Self::with_sink(
            intent_classifier,
            shallow,
            clarifier,
            deep_orchestrator,
            planner,
            researcher,
            session_store,
            source_sink,
            vector_store,
        ))
    }
}
