use crate::domain::models::log_entry::AgentTag;
use crate::domain::traits::agent::MuonAgent;
use crate::domain::traits::session_store::SessionStore;
use crate::error::MuonError;

pub struct InfrastructureContext {
    pub intent_classifier: Box<dyn MuonAgent>,
    pub shallow: Box<dyn MuonAgent>,
    pub clarifier: Box<dyn MuonAgent>,
    pub deep_orchestrator: Box<dyn MuonAgent>,
    pub planner: Box<dyn MuonAgent>,
    pub researcher: Box<dyn MuonAgent>,
    pub session_store: Box<dyn SessionStore>,
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
        Self {
            intent_classifier,
            shallow,
            clarifier,
            deep_orchestrator,
            planner,
            researcher,
            session_store,
        }
    }

    pub async fn new_live(
        cfg: &crate::config::MuonConfig,
        bridge: &crate::application::bridge::BridgeChannels,
    ) -> Result<Self, MuonError> {
        use rig_core::client::CompletionClient;
        use rig_core::providers::openai;

        let provider = &cfg.agents.intent_classifier.provider;
        if provider != "openai" {
            return Err(MuonError::Config(format!(
                "unsupported provider '{}'",
                provider
            )));
        }

        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            MuonError::Config("OPENAI_API_KEY not set".to_string())
        })?;
        let client = openai::Client::new(api_key)
            .map_err(|e| MuonError::Config(e.to_string()))?;

        let factory =
            crate::infrastructure::agent_rs::ReActFactory::new(
                cfg,
                bridge.clone(),
            );
        let preamble = &cfg.advanced.agent_preamble;

        let intent_agent = client
            .agent(&cfg.agents.intent_classifier.model)
            .preamble(preamble)
            .build();
        let intent_classifier: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Intent,
                factory.build_runner(
                    intent_agent,
                    AgentTag::Intent,
                    5,
                    cfg.agents.intent_classifier.timeout_sec,
                ),
                bridge.clone(),
            ),
        );

        let shallow_agent = client
            .agent(&cfg.agents.shallow_researcher.model)
            .preamble(preamble)
            .build();
        let shallow: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Search,
                factory.build_runner(
                    shallow_agent,
                    AgentTag::Search,
                    cfg.agents.shallow_researcher.max_tool_iters
                        as usize,
                    30,
                ),
                bridge.clone(),
            ),
        );

        let clarifier_agent = client
            .agent(&cfg.agents.clarifier.model)
            .preamble(preamble)
            .build();
        let compaction_threshold =
            (cfg.advanced.compaction_threshold * 100.0) as usize;
        let clarifier: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Clarify,
                factory.build_clarifier_runner(
                    clarifier_agent,
                    AgentTag::Clarify,
                    cfg.agents.clarifier.max_iterations as usize,
                    30,
                    compaction_threshold,
                ),
                bridge.clone(),
            ),
        );

        let deep_agent = client
            .agent(&cfg.agents.deep_researcher.orchestrator.model)
            .preamble(preamble)
            .build();
        let deep_orchestrator: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Orchestrate,
                factory.build_runner(
                    deep_agent,
                    AgentTag::Orchestrate,
                    cfg.agents.deep_researcher.iterations as usize,
                    30,
                ),
                bridge.clone(),
            ),
        );

        let planner_agent = client
            .agent(&cfg.agents.deep_researcher.planner.model)
            .preamble(preamble)
            .build();
        let planner: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Plan,
                factory.build_planner_runner(
                    planner_agent,
                    AgentTag::Plan,
                    cfg.agents.deep_researcher.iterations as usize,
                    30,
                ),
                bridge.clone(),
            ),
        );

        let researcher_agent = client
            .agent(&cfg.agents.deep_researcher.researcher.model)
            .preamble(preamble)
            .build();
        let researcher: Box<dyn MuonAgent> = Box::new(
            crate::infrastructure::agent_rs::ReActAgent::new(
                AgentTag::Search,
                factory.build_researcher_runner(
                    researcher_agent,
                    AgentTag::Search,
                    cfg.agents.deep_researcher.iterations as usize,
                    30,
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

        Ok(Self::new(
            intent_classifier,
            shallow,
            clarifier,
            deep_orchestrator,
            planner,
            researcher,
            session_store,
        ))
    }
}
