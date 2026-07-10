use crate::presentation::views::RenderParams;

use super::state::AppState;
use super::types::ActivePopup;

pub(crate) fn render(frame: &mut ratatui::Frame, app: &mut AppState) {
    app.term_cols = frame.area().width;
    app.term_rows = frame.area().height;
    app.hit_registry.clear();
    let view = app.router.active();
    let clarifier_question = app.clarifier_pending.as_ref().map(|c| c.question.as_str());
    let clarifier_response = app.clarifier_response.as_str();
    let clarifier_log = if app.last_clarifier_log.is_empty() {
        None
    } else {
        Some(app.last_clarifier_log.as_str())
    };
    view.render(frame, frame.area(), &mut RenderParams {
        query_input: &app.query_input,
        sessions: app.sessions.list(),
        pipeline: &app.pipeline,
        config: &app.config,
        forms: &app.forms,
        settings_tab: app.router.settings_tab(),
        hit_registry: &mut app.hit_registry,
        mouse_col: app.mouse_col,
        mouse_row: app.mouse_row,
        clarifier_question,
        clarifier_response,
        last_report: app.last_report.as_ref(),
        last_sources: app.last_sources.as_slice(),
        live_feed: app.live_feed_entries.as_slice(),
        live_feed_scroll: app.live_feed_scroll,
        clarifier_log,
        clarifier_focused: app.clarifier_focused,
        report_scroll: app.report_scroll,
        source_scroll: app.source_scroll,
        session_scroll: app.session_scroll,
        full_report_mode: app.full_report_mode,
        term_cols: app.term_cols,
        term_rows: app.term_rows,
    });

    if let Some(popup) = &app.active_popup {
        match popup {
            ActivePopup::EditModels { provider_idx, focus_idx, edit_buffer, edit_cursor, scroll_offset } => {
                crate::presentation::components::inputs::settings::providers::render_models_popup(
                    frame,
                    frame.area(),
                    &app.config,
                    *provider_idx,
                    *focus_idx,
                    *scroll_offset,
                    edit_buffer.as_deref(),
                    *edit_cursor,
                    &mut app.hit_registry,
                    app.mouse_col,
                    app.mouse_row,
                );
            }
            ActivePopup::ConfigureSearch { provider_idx, focus_idx, edit_buffer, edit_cursor } => {
                crate::presentation::components::inputs::settings::tools::render_configure_popup(
                    frame,
                    frame.area(),
                    &app.config,
                    *provider_idx,
                    *focus_idx,
                    edit_buffer.as_deref(),
                    *edit_cursor,
                    &mut app.hit_registry,
                    app.mouse_col,
                    app.mouse_row,
                );
            }
            ActivePopup::PlanApproval { plan, responder: _, focus, feedback_buffer, feedback_cursor } => {
                crate::presentation::components::panels::plan_approval::render(
                    frame,
                    frame.area(),
                    plan,
                    *focus,
                    feedback_buffer,
                    *feedback_cursor,
                    &mut app.hit_registry,
                    app.mouse_col,
                    app.mouse_row,
                );
            }
        }
    }

    if let Some((_, msg, kind)) = &app.status_flash {
        crate::presentation::components::chrome::toast::render(frame, frame.area(), msg, *kind);
    }
}

