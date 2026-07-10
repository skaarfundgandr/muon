use crossterm::event::{KeyCode, KeyEvent};

use crate::presentation::{ActivePopup, AppState, PlanApprovalFocus};
use crate::application::bridge::PlanDecision;
use crate::presentation::handlers::{dashboard, settings, view_events};
use crate::presentation::views::View;

pub fn handle_key(app: &mut AppState, key: KeyEvent) {
    if let Some(ActivePopup::PlanApproval {
        focus,
        feedback_buffer,
        feedback_cursor,
        ..
    }) = &mut app.active_popup
    {
        match key.code {
            KeyCode::Tab | KeyCode::Right | KeyCode::Down => {
                *focus = match *focus {
                    PlanApprovalFocus::Approve => PlanApprovalFocus::Reject,
                    PlanApprovalFocus::Reject => PlanApprovalFocus::Feedback,
                    PlanApprovalFocus::Feedback => PlanApprovalFocus::Approve,
                };
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Up => {
                *focus = match *focus {
                    PlanApprovalFocus::Approve => PlanApprovalFocus::Feedback,
                    PlanApprovalFocus::Reject => PlanApprovalFocus::Approve,
                    PlanApprovalFocus::Feedback => PlanApprovalFocus::Reject,
                };
            }
            KeyCode::Esc => {
                if let Some(ActivePopup::PlanApproval { responder, .. }) = app.active_popup.take() {
                    let _ = responder.send(PlanDecision::Reject);
                }
            }
            KeyCode::Enter => match *focus {
                PlanApprovalFocus::Approve => {
                    if let Some(ActivePopup::PlanApproval { responder, .. }) =
                        app.active_popup.take()
                    {
                        let _ = responder.send(PlanDecision::Approve);
                    }
                }
                PlanApprovalFocus::Reject => {
                    if let Some(ActivePopup::PlanApproval { responder, .. }) =
                        app.active_popup.take()
                    {
                        let _ = responder.send(PlanDecision::Reject);
                    }
                }
                PlanApprovalFocus::Feedback => {
                    if let Some(ActivePopup::PlanApproval {
                        responder,
                        feedback_buffer,
                        ..
                    }) = app.active_popup.take()
                    {
                        let _ = responder.send(PlanDecision::Feedback(feedback_buffer));
                    }
                }
            },
            _ => {
                if *focus == PlanApprovalFocus::Feedback {
                    match key.code {
                        KeyCode::Char(c) => {
                            feedback_buffer.insert(*feedback_cursor, c);
                            *feedback_cursor += 1;
                        }
                        KeyCode::Backspace if *feedback_cursor > 0 => {
                            *feedback_cursor -= 1;
                            feedback_buffer.remove(*feedback_cursor);
                        }
                        _ => {}
                    }
                }
            }
        }
        return;
    }

    let view = app.router.active();

    match view {
        View::Settings => {
            settings::handle(app, key);
        }
        View::Dashboard => {
            dashboard::handle(app, key);
        }
        _ => {
            view_events::handle(app, key);
        }
    }
}
