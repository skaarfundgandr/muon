use crate::application::pipeline::PipelineStage;
use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::SessionId;
use crate::domain::models::source::Source;
use crate::domain::traits::session_store::SessionStore;

pub async fn finalize_session(
    store: &dyn SessionStore,
    session_id: SessionId,
    report: &ResearchReport,
    sources: &[Source],
    stage: PipelineStage,
) -> Result<(), MuonError> {
    store.save_report(session_id, report).await?;
    store.save_sources(session_id, sources).await?;
    store.update_stage(session_id, stage.as_str()).await?;
    Ok(())
}
