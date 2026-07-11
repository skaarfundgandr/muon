use std::time::Instant;

use crate::application::bridge::PlanDecision;
use crate::presentation::click::ClickAction;
use crate::presentation::views::{SettingsTab, View};

use crate::presentation::state::AppState;
use crate::presentation::types::{ActivePopup, Event, PlanApprovalFocus};

pub(crate) fn handle_mouse_click(app: &mut AppState, col: u16, row: u16) {
    for target in app.hit_registry.iter().rev() {
        if !crate::presentation::click::is_hovering(target.rect, col, row) {
            continue;
        }
        let mut close_popup_decision = None;
        if let Some(popup) = &mut app.active_popup {
            match popup {
                ActivePopup::EditModels { provider_idx, focus_idx, edit_buffer, edit_cursor, scroll_offset } => {
                    match &target.action {
                        ClickAction::ActivateField(idx) => {
                            *focus_idx = *idx;
                            *edit_buffer = None;
                            *edit_cursor = 0;
                            let model_idx = idx / 3;
                            let sub_idx = idx % 3;
                            if model_idx < app.config.providers[*provider_idx].models.len() {
                                let current = match sub_idx {
                                    0 => app.config.providers[*provider_idx].models[model_idx].name.clone(),
                                    1 => app.config.providers[*provider_idx].models[model_idx].model_id.clone(),
                                    _ => String::new(),
                                };
                                if sub_idx < 2 {
                                    *edit_buffer = Some(current.clone());
                                    *edit_cursor = current.len();
                                }
                            }
                            
                            // Adjust scroll_offset to keep focused model in view
                            let popup_h = 18u16.min(app.term_rows);
                            let inner_h = popup_h.saturating_sub(2);
                            let chunks_0_h = inner_h.saturating_sub(2);
                            let max_visible_models = ((chunks_0_h / 2) as usize).max(1);
                            if model_idx < *scroll_offset {
                                *scroll_offset = model_idx;
                            } else if model_idx >= *scroll_offset + max_visible_models {
                                *scroll_offset = model_idx + 1 - max_visible_models;
                            }
                            return;
                        }
                        ClickAction::RemoveModel(idx) => {
                            if *idx < app.config.providers[*provider_idx].models.len() {
                                app.config.providers[*provider_idx].models.remove(*idx);
                                *focus_idx = 0;
                                *scroll_offset = 0;
                                app.forms[SettingsTab::Providers as usize].dirty = true;
                            }
                            return;
                        }
                        ClickAction::AddModel => {
                            app.config.providers[*provider_idx].models.push(crate::config::ProviderModel {
                                name: String::new(),
                                model_id: String::new(),
                                description: String::new(),
                            });
                            let m = app.config.providers[*provider_idx].models.len();
                            *focus_idx = 3 * m - 3;
                            let popup_h = 18u16.min(app.term_rows);
                            let inner_h = popup_h.saturating_sub(2);
                            let chunks_0_h = inner_h.saturating_sub(2);
                            let max_visible_models = ((chunks_0_h / 2) as usize).max(1);
                            if m > max_visible_models {
                                *scroll_offset = m - max_visible_models;
                            } else {
                                *scroll_offset = 0;
                            }
                            app.forms[SettingsTab::Providers as usize].dirty = true;
                            return;
                        }
                        ClickAction::SwitchView(View::Settings) => {
                            app.active_popup = None;
                            return;
                        }
                        _ => {}
                    }
                }
                ActivePopup::ConfigureSearch { provider_idx, focus_idx, edit_buffer, edit_cursor } => {
                    match &target.action {
                        ClickAction::ActivateField(idx) => {
                            *focus_idx = *idx;
                            *edit_buffer = None;
                            *edit_cursor = 0;
                            if *idx < 3 {
                                let current = if *idx == 0 {
                                    app.config.search.providers[*provider_idx].name.clone()
                                } else if *idx == 1 {
                                    app.config.search.providers[*provider_idx].api_key.clone()
                                } else {
                                    app.config.search.providers[*provider_idx].max_results.map(|x| x.to_string()).unwrap_or_default()
                                };
                                *edit_buffer = Some(current.clone());
                                *edit_cursor = current.len();
                            }
                            return;
                        }
                        ClickAction::SwitchView(View::Settings) => {
                            app.active_popup = None;
                            return;
                        }
                        _ => {}
                    }
                }
                ActivePopup::PlanApproval { focus, .. } => {
                    match &target.action {
                        ClickAction::PlanApprove => {
                            close_popup_decision = Some(PlanDecision::Approve);
                        }
                        ClickAction::PlanReject => {
                            close_popup_decision = Some(PlanDecision::Reject);
                        }
                        ClickAction::PlanFeedback => {
                            if let ActivePopup::PlanApproval { feedback_buffer, .. } = popup {
                                close_popup_decision = Some(PlanDecision::Feedback(feedback_buffer.clone()));
                            }
                        }
                        ClickAction::PlanSelectFeedbackInput => {
                            *focus = PlanApprovalFocus::Feedback;
                            return;
                        }
                        _ => {}
                    }
                }
            }
            if close_popup_decision.is_none() {
                return;
            }
        }

        if let Some(decision) = close_popup_decision {
            if let Some(ActivePopup::PlanApproval { responder, .. }) = app.active_popup.take() {
                let _ = responder.send(decision);
            }
            return;
        }

        match &target.action {
            ClickAction::ActivateField(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                app.forms[tab_idx].focus = *idx;
                app.forms[tab_idx].reset_edit();
                let current = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::get_field(&app.config, *idx),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::get_field(&app.config.agents, *idx),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::get_field(&app.config, *idx),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::get_field(&app.config, *idx),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::get_field(&app.config.display, *idx),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::get_field(&app.config.advanced, *idx),
                };
                let kind = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields(&app.config)[*idx].kind,
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::fields()[*idx].kind,
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields(&app.config)[*idx].kind,
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields(&app.config)[*idx].kind,
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::fields()[*idx].kind,
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::fields()[*idx].kind,
                };
                match kind {
                    crate::presentation::form::FieldKind::Text | crate::presentation::form::FieldKind::Number => {
                        app.forms[tab_idx].begin_edit(&current);
                    }
                    crate::presentation::form::FieldKind::Dropdown => {
                        if app.forms[tab_idx].dropdown_open {
                            app.forms[tab_idx].dropdown_open = false;
                        } else {
                            app.forms[tab_idx].open_dropdown();
                        }
                    }
                    crate::presentation::form::FieldKind::Checkbox => {
                        match tab {
                            SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::toggle_field(&mut app.config, *idx),
                            SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::toggle_field(&mut app.config.agents, *idx),
                            SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::toggle_field(&mut app.config, *idx),
                            SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::toggle_field(&mut app.config, *idx),
                            SettingsTab::Display => crate::presentation::components::inputs::settings::display::toggle_field(&mut app.config.display, *idx),
                            SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::toggle_field(&mut app.config.advanced, *idx),
                        }
                        app.forms[tab_idx].dirty = true;
                    }
                    crate::presentation::form::FieldKind::Button => {
                        match tab {
                            SettingsTab::DataSources => {
                                if *idx == 6 {
                                    let path = app.config.data_sources.source_path.clone();
                                    let kind = app.config.data_sources.source_type.to_uppercase();
                                    app.config.data_sources.rag_indexes.push(crate::config::RagIndexConfig {
                                        path,
                                        kind,
                                        status: "○ pending".to_string(),
                                        chunks: "0".to_string(),
                                    });
                                    app.forms[tab_idx].dirty = true;
                                }
                            }
                            SettingsTab::Providers => {
                                let n = app.config.providers.len();
                                if *idx == 5 * n {
                                    app.config.providers.push(crate::config::ProviderConfig {
                                        name: String::new(),
                                        base_url: String::new(),
                                        api_key: String::new(),
                                        models: Vec::new(),
                                        provider_type: crate::config::ProviderType::OpenAICompatible,
                                    });
                                    app.forms[tab_idx].focus = 5 * app.config.providers.len() - 5;
                                    app.forms[tab_idx].dirty = true;
                                }
                            }
                            SettingsTab::Tools => {
                                let n = app.config.search.providers.len();
                                if *idx == 5 * n + 1 {
                                    app.config.search.providers.push(crate::config::SearchProviderConfig {
                                        name: String::new(),
                                        provider_type: crate::config::SearchProviderType::Tavily,
                                        api_key: String::new(),
                                        max_results: None,
                                        tavily: Default::default(),
                                        firecrawl: Default::default(),
                                        brave: Default::default(),
                                        serper: Default::default(),
                                    });
                                    app.forms[tab_idx].focus = 5 * app.config.search.providers.len() - 5;
                                    app.forms[tab_idx].dirty = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ClickAction::ToggleCheckbox(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                app.forms[tab_idx].focus = *idx;
                app.forms[tab_idx].reset_edit();
                match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::toggle_field(&mut app.config, *idx),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::toggle_field(&mut app.config.agents, *idx),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::toggle_field(&mut app.config, *idx),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::toggle_field(&mut app.config, *idx),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::toggle_field(&mut app.config.display, *idx),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::toggle_field(&mut app.config.advanced, *idx),
                }
                app.forms[tab_idx].dirty = true;
            }
            ClickAction::FocusField(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                app.forms[tab_idx].focus = *idx;
                app.forms[tab_idx].reset_edit();
            }
            ClickAction::SwitchSettingsTab(tab) => {
                app.router.set_settings_tab(*tab);
                let new_idx = *tab as usize;
                app.forms[new_idx].reset_edit();
                app.forms[new_idx].focus = 0;
            }
            ClickAction::SwitchView(v) => {
                app.router.transition(*v);
                if *v == View::Settings {
                    let tab_idx = app.router.settings_tab() as usize;
                    app.forms[tab_idx].focus = 0;
                    app.forms[tab_idx].reset_edit();
                }
            }
            ClickAction::ActivateQueryInput => {
                app.query_input.active = true;
                app.clarifier_focused = false;
            }
            ClickAction::SelectSession(idx) => {
                app.restore_session(*idx);
            }
            ClickAction::DeleteSession(idx) => {
                app.delete_session_at(*idx);
            }
            ClickAction::SelectDropdownOption(idx) => {
                let tab = app.router.settings_tab();
                let tab_idx = tab as usize;
                let options: Vec<String> = match tab {
                    SettingsTab::Providers => crate::presentation::components::inputs::settings::providers::fields(&app.config)[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Agents => crate::presentation::components::inputs::settings::agents::options_for(
                        app.forms[tab_idx].focus,
                        &app.config,
                    ),
                    SettingsTab::Tools => crate::presentation::components::inputs::settings::tools::fields(&app.config)[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::DataSources => crate::presentation::components::inputs::settings::data_sources::fields(&app.config)[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Display => crate::presentation::components::inputs::settings::display::fields()[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    SettingsTab::Advanced => crate::presentation::components::inputs::settings::advanced::fields()[app.forms[tab_idx].focus]
                        .options
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                };
                if *idx < options.len() {
                    let val = options[*idx].clone();
                    if val.starts_with("<no models") {
                        app.forms[tab_idx].dropdown_open = false;
                        return;
                    }
                    app.forms[tab_idx].dropdown_open = false;
                    match tab {
                        SettingsTab::Providers => {
                            crate::presentation::components::inputs::settings::providers::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Agents => {
                            crate::presentation::components::inputs::settings::agents::set_field(&mut app.config.agents, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Tools => {
                            crate::presentation::components::inputs::settings::tools::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::DataSources => {
                            crate::presentation::components::inputs::settings::data_sources::set_field(&mut app.config, app.forms[tab_idx].focus, &val);
                        }
                        SettingsTab::Display => {
                            crate::presentation::components::inputs::settings::display::set_field(&mut app.config.display, app.forms[tab_idx].focus, &val);
                            if app.forms[tab_idx].focus == 0
                                && let Some(palette) = crate::presentation::theme::for_name(&val)
                            {
                                crate::presentation::theme::replace(palette);
                            }
                        }
                        SettingsTab::Advanced => {
                            crate::presentation::components::inputs::settings::advanced::set_field(&mut app.config.advanced, app.forms[tab_idx].focus, &val);
                        }
                    }
                    app.forms[tab_idx].dirty = true;
                }
            }
            ClickAction::ActivateClarifier => {
                app.query_input.active = false;
                app.clarifier_focused = true;
            }
            ClickAction::AddProvider => {
                use crate::config::{ProviderConfig, ProviderType};
                app.config.providers.push(ProviderConfig {
                    name: String::new(),
                    base_url: String::new(),
                    api_key: String::new(),
                    models: Vec::new(),
                    provider_type: ProviderType::OpenAICompatible,
                });
                app.forms[SettingsTab::Providers as usize].dirty = true;
            }
            ClickAction::RemoveProvider(idx) => {
                if *idx < app.config.providers.len() {
                    app.config.providers.remove(*idx);
                    let form = &mut app.forms[SettingsTab::Providers as usize];
                    form.dirty = true;
                    let n = app.config.providers.len();
                    if n == 0 {
                        form.focus = 0;
                    } else if form.focus >= 5 * n {
                        form.focus = 5 * n - 5;
                    } else {
                        let p = form.focus / 5;
                        if p >= n {
                            form.focus = 5 * (n - 1);
                        }
                    }
                }
            }
            ClickAction::AddSearchProvider => {
                use crate::config::{SearchProviderConfig, SearchProviderType};
                app.config.search.providers.push(SearchProviderConfig {
                    name: String::new(),
                    provider_type: SearchProviderType::Tavily,
                    api_key: String::new(),
                    max_results: None,
                    tavily: Default::default(),
                    firecrawl: Default::default(),
                    brave: Default::default(),
                    serper: Default::default(),
                });
                app.forms[SettingsTab::Tools as usize].dirty = true;
            }
            ClickAction::RemoveSearchProvider(idx) => {
                if *idx < app.config.search.providers.len() {
                    app.config.search.providers.swap_remove(*idx);
                    app.forms[SettingsTab::Tools as usize].dirty = true;
                }
            }
            ClickAction::ToggleSearchProvider(_idx) => {
                // No per-provider enabled flag yet — UI hint only.
            }
            ClickAction::ToggleArxiv => {
                app.config.search.papers.arxiv_enabled = !app.config.search.papers.arxiv_enabled;
                app.forms[SettingsTab::Tools as usize].dirty = true;
            }
            ClickAction::EditProviderModels(idx) => {
                app.active_popup = Some(ActivePopup::EditModels {
                    provider_idx: *idx,
                    focus_idx: 0,
                    edit_buffer: None,
                    edit_cursor: 0,
                    scroll_offset: 0,
                });
            }
            ClickAction::FetchProviderModels(idx) => {
                let idx = *idx;
                if idx >= app.config.providers.len() {
                    return;
                }
                let provider = &app.config.providers[idx];
                let api_key = match provider.resolved_api_key() {
                    Ok(k) => k,
                    Err(e) => {
                        app.status_flash = Some((
                            Instant::now(),
                            format!("API key error: {e}"),
                            crate::presentation::components::chrome::toast::ToastKind::Error,
                        ));
                        return;
                    }
                };
                let mut base_url = provider.base_url.trim().to_string();
                if base_url.ends_with('/') {
                    base_url.pop();
                }
                let url = format!("{base_url}/models");
                let Some(tx) = app.event_tx.clone() else {
                    return;
                };
                app.status_flash = Some((
                    Instant::now(),
                    "Fetching models…".into(),
                    crate::presentation::components::chrome::toast::ToastKind::Info,
                ));
                tokio::spawn(async move {
                    let client = reqwest::Client::new();
                    let mut req = client.get(&url);
                    if !api_key.is_empty() {
                        req = req.header("Authorization", format!("Bearer {api_key}"));
                    }
                    let result = match req.send().await {
                        Ok(resp) => {
                            #[derive(serde::Deserialize)]
                            struct ModelData {
                                id: String,
                            }
                            #[derive(serde::Deserialize)]
                            struct ModelsResponse {
                                data: Vec<ModelData>,
                            }
                            match resp.json::<ModelsResponse>().await {
                                Ok(parsed) => Ok(parsed
                                    .data
                                    .into_iter()
                                    .map(|m| crate::config::ProviderModel {
                                        name: m.id.clone(),
                                        model_id: m.id,
                                        description: String::new(),
                                    })
                                    .collect()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = tx.send(Event::ModelsFetched {
                        provider_index: idx,
                        result,
                    });
                });
            }
            ClickAction::ConfigureSearchOptions(idx) => {
                app.active_popup = Some(ActivePopup::ConfigureSearch {
                    provider_idx: *idx,
                    focus_idx: 0,
                    edit_buffer: None,
                    edit_cursor: 0,
                });
            }
            ClickAction::ReindexRagIndex(idx) if *idx < app.config.data_sources.rag_indexes.len() => {
                app.status_flash = Some((
                    Instant::now(),
                    "RAG reindex is not implemented yet".into(),
                    crate::presentation::components::chrome::toast::ToastKind::Info,
                ));
            }
            ClickAction::RemoveRagIndex(idx) if *idx < app.config.data_sources.rag_indexes.len() => {
                app.config.data_sources.rag_indexes.remove(*idx);
                app.forms[SettingsTab::DataSources as usize].dirty = true;
            }
            ClickAction::ExportMarkdown => {
                app.action_export_markdown();
            }
            ClickAction::SyncObsidian => {
                app.action_sync_obsidian();
            }
            ClickAction::NewQuery => {
                app.action_new_query();
            }
            ClickAction::RefineSearch => {
                app.action_refine_search();
            }
            ClickAction::FullReportView => {
                app.action_toggle_full_report();
            }
            ClickAction::CopySourceUrl(url) => {
                let url = url.clone();
                app.action_copy_source_url(&url);
            }
            _ => {}
        }
        return;
    }
}

