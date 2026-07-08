use crate::domain::models::log_entry::AgentTag;
use crate::domain::traits::agent::MuonAgent;
use crate::domain::traits::session_store::SessionStore;
use crate::error::MuonError;
use crate::infrastructure::source_registry::SourceRegistry;
use std::sync::{Arc, Mutex};

pub struct InfrastructureContext {
    pub intent_classifier: Box<dyn MuonAgent>,
    pub shallow: Box<dyn MuonAgent>,
    pub clarifier: Box<dyn MuonAgent>,
    pub deep_orchestrator: Box<dyn MuonAgent>,
    pub planner: Box<dyn MuonAgent>,
    pub researcher: Box<dyn MuonAgent>,
    pub session_store: Box<dyn SessionStore>,
    pub source_sink: Arc<Mutex<SourceRegistry>>,
    pub vector_store: Option<Box<dyn crate::domain::traits::vector_store::VectorStore>>,
}

impl std::fmt::Debug for InfrastructureContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfrastructureContext").finish_non_exhaustive()
    }
}

impl InfrastructureContext {
    pub fn new(
        intent_classifier: Box<dyn MuonAgent>,
        shallow: Box<dyn MuonAgent>,
        clarifier: Box<dyn MuonAgent>,
        deep_orchestrator: Box<dyn MuonAgent>,
        planner: Box<dyn MuonAgent>,
        researcher: Box<dyn MuonAgent>,
        session_store: Box<dyn SessionStore>,
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
        intent_classifier: Box<dyn MuonAgent>,
        shallow: Box<dyn MuonAgent>,
        clarifier: Box<dyn MuonAgent>,
        deep_orchestrator: Box<dyn MuonAgent>,
        planner: Box<dyn MuonAgent>,
        researcher: Box<dyn MuonAgent>,
        session_store: Box<dyn SessionStore>,
        source_sink: Arc<Mutex<SourceRegistry>>,
        vector_store: Option<Box<dyn crate::domain::traits::vector_store::VectorStore>>,
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
        cfg: &crate::config::MuonConfig,
        bridge: &crate::application::bridge::BridgeChannels,
    ) -> Result<Self, MuonError> {
        use rig_core::client::CompletionClient;
        use std::sync::Arc;
        use crate::infrastructure::providers::ProviderClient;

        let factory =
            crate::infrastructure::agent_rs::ReActFactory::new(
                cfg,
                bridge.clone(),
            );
        let preamble = &cfg.advanced.agent_preamble;
        let source_sink = Arc::new(Mutex::new(SourceRegistry::new()));

        let providers = &cfg.providers;
        fn resolve_model_id(providers: &[crate::config::ProviderConfig], provider_name: &str, model_name: &str) -> String {
            providers
                .iter()
                .find(|p| p.name == provider_name)
                .and_then(|p| p.models.iter().find(|m| m.name == model_name))
                .and_then(|m| if m.model_id.is_empty() { None } else { Some(m.model_id.clone()) })
                .unwrap_or_else(|| model_name.to_string())
        }
        let resolve_client = |name: &str| {
            if name.is_empty() {
                crate::infrastructure::providers::ResolvedClient::for_default_provider(providers)
            } else {
                crate::infrastructure::providers::ResolvedClient::for_named_provider(name, providers)
            }
        };
        let intent_client = resolve_client(&cfg.agents.intent_classifier.provider)?;
        let shallow_client = resolve_client(&cfg.agents.shallow_researcher.provider)?;
        let clarifier_client = resolve_client(&cfg.agents.clarifier.provider)?;
        let deep_client = resolve_client(&cfg.agents.deep_researcher.orchestrator.provider)?;
        let planner_client = resolve_client(&cfg.agents.deep_researcher.planner.provider)?;
        let researcher_client = resolve_client(&cfg.agents.deep_researcher.researcher.provider)?;

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

        // Intent Classifier — no tools.
        let intent_classifier: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Intent,
                build_agent!(
                    &intent_client.client,
                    &resolve_model_id(providers, &cfg.agents.intent_classifier.provider, &cfg.agents.intent_classifier.model),
                    |c| {
                        let ic_preamble = "You are an intent classifier. Classify the user's query and respond \
                            with STRICT JSON only — no other text, no markdown, no explanation:\n\
                            {\"intent\": \"research\"|\"meta\", \"depth\": \"shallow\"|\"deep\"}\n\
                            If intent is \"meta\", also include a \"response\" field with a direct answer.";

                        let agent = c
                            .agent(resolve_model_id(providers, &cfg.agents.intent_classifier.provider, &cfg.agents.intent_classifier.model))
                            .preamble(ic_preamble)
                            .build();
                        factory.build_runner(
                            agent,
                            AgentTag::Intent,
                            5,
                            cfg.agents.intent_classifier.timeout_sec,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        // Shallow Researcher — fetch_page (always), web_search (if configured).
        let shallow: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Search,
                build_agent!(
                    &shallow_client.client,
                    &resolve_model_id(providers, &cfg.agents.shallow_researcher.provider, &cfg.agents.shallow_researcher.model),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(providers, &cfg.agents.shallow_researcher.provider, &cfg.agents.shallow_researcher.model))
                            .preamble(preamble)
                            .tool(crate::infrastructure::agent_rs::tools::FetchPageTool::new(
                                fetch_http.clone(),
                                8000,
                            ));
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(wp.clone()))
                        } else {
                            b
                        };
                        let agent = b
                            .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                            .build();
                        factory.build_runner(
                            agent,
                            AgentTag::Search,
                            cfg.agents.shallow_researcher.max_tool_iters
                                as usize,
                            30,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        // Clarifier — web_search (if configured).
        let compaction_threshold =
            (cfg.advanced.compaction_threshold * 100.0) as usize;
        let clarifier: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Clarify,
                build_agent!(
                    &clarifier_client.client,
                    &resolve_model_id(providers, &cfg.agents.clarifier.provider, &cfg.agents.clarifier.model),
                    |c| {
                        let clarifier_preamble = "You are a clarifying and planning agent. Ask clarifying questions to resolve ambiguities in the user's query, and construct a detailed research plan. You must respond with strict JSON only.";
                        let agent = if let Some(ref wp) = web_provider {
                            c.agent(resolve_model_id(providers, &cfg.agents.clarifier.provider, &cfg.agents.clarifier.model))
                                .preamble(clarifier_preamble)
                                .tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(wp.clone()))
                                .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                                .build()
                        } else {
                            c.agent(resolve_model_id(providers, &cfg.agents.clarifier.provider, &cfg.agents.clarifier.model))
                                .preamble(clarifier_preamble)
                                .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                                .build()
                        };
                        factory.build_clarifier_runner(
                            agent,
                            AgentTag::Clarify,
                            cfg.agents.clarifier.max_iterations as usize,
                            30,
                            compaction_threshold,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        // Deep Orchestrator — think (always), web_search + paper_search (if configured).
        let deep_orchestrator: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Orchestrate,
                build_agent!(
                    &deep_client.client,
                    &resolve_model_id(providers, &cfg.agents.deep_researcher.orchestrator.provider, &cfg.agents.deep_researcher.orchestrator.model),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(providers, &cfg.agents.deep_researcher.orchestrator.provider, &cfg.agents.deep_researcher.orchestrator.model))
                            .preamble(preamble)
                            .tool(crate::infrastructure::agent_rs::tools::ThinkTool);
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(wp.clone()))
                        } else {
                            b
                        };
                        let b = if !paper_providers.is_empty() {
                            b.tool(crate::infrastructure::agent_rs::tools::PaperSearchTool::new(
                                paper_providers.clone(),
                            ))
                        } else {
                            b
                        };
                        let agent = b
                            .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                            .build();
                        factory.build_runner(
                            agent,
                            AgentTag::Orchestrate,
                            cfg.agents.deep_researcher.iterations as usize,
                            30,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        // Planner — think (always), web_search + paper_search (if configured).
        let planner: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Plan,
                build_agent!(
                    &planner_client.client,
                    &resolve_model_id(providers, &cfg.agents.deep_researcher.planner.provider, &cfg.agents.deep_researcher.planner.model),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(providers, &cfg.agents.deep_researcher.planner.provider, &cfg.agents.deep_researcher.planner.model))
                            .preamble(preamble)
                            .tool(crate::infrastructure::agent_rs::tools::ThinkTool);
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(wp.clone()))
                        } else {
                            b
                        };
                        let b = if !paper_providers.is_empty() {
                            b.tool(crate::infrastructure::agent_rs::tools::PaperSearchTool::new(
                                paper_providers.clone(),
                            ))
                        } else {
                            b
                        };
                        let agent = b
                            .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                            .build();
                        factory.build_planner_runner(
                            agent,
                            AgentTag::Plan,
                            cfg.agents.deep_researcher.iterations as usize,
                            30,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        // Researcher — think + fetch_page (always), web_search + paper_search (if configured).
        let researcher: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Search,
                build_agent!(
                    &researcher_client.client,
                    &resolve_model_id(providers, &cfg.agents.deep_researcher.researcher.provider, &cfg.agents.deep_researcher.researcher.model),
                    |c| {
                        let b = c
                            .agent(resolve_model_id(providers, &cfg.agents.deep_researcher.researcher.provider, &cfg.agents.deep_researcher.researcher.model))
                            .preamble(preamble)
                            .tool(crate::infrastructure::agent_rs::tools::ThinkTool)
                            .tool(crate::infrastructure::agent_rs::tools::FetchPageTool::new(
                                fetch_http.clone(),
                                8000,
                            ));
                        let b = if let Some(ref wp) = web_provider {
                            b.tool(crate::infrastructure::agent_rs::tools::WebSearchTool::new(wp.clone()))
                        } else {
                            b
                        };
                        let b = if !paper_providers.is_empty() {
                            b.tool(crate::infrastructure::agent_rs::tools::PaperSearchTool::new(
                                paper_providers.clone(),
                            ))
                        } else {
                            b
                        };
                        let agent = b
                            .default_max_turns(cfg.advanced.max_tool_calls_per_turn as usize)
                            .build();
                        factory.build_researcher_runner(
                            agent,
                            AgentTag::Search,
                            cfg.agents.deep_researcher.iterations as usize,
                            30,
                            source_sink.clone(),
                        )
                    }
                ),
                bridge.clone(),
            ),
        );

        let pool = crate::infrastructure::storage::init_pool(
            &cfg.advanced.session_db_path,
        )
        .await?;
        let session_store: Box<dyn SessionStore> =
            Box::new(
                crate::infrastructure::storage::DieselSessionStore::new(
                    pool,
                ),
            );

        // RAG — optional, fails open (no RAG) if the model/index can't init.
        let vector_store: Option<Box<dyn crate::domain::traits::vector_store::VectorStore>> =
            match crate::infrastructure::rag::RagContext::open(cfg).await {
                Ok(rag) => {
                    use crate::domain::models::log_entry::{AgentTag, LogLevel};
                    bridge.log(AgentTag::Sys, LogLevel::Info, "RAG context initialized");
                    Some(Box::new(rag))
                }
                Err(e) => {
                    use crate::domain::models::log_entry::{AgentTag, LogLevel};
                    bridge.log(AgentTag::Sys, LogLevel::Warn, format!("RAG init failed, continuing without: {e}"));
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
