use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: Uuid,
    pub title: String,
    pub query: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Default)]
pub struct SessionService {
    sessions: Vec<SessionSummary>,
}

impl SessionService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> &[SessionSummary] {
        &self.sessions
    }

    pub fn create(&mut self, query: &str) -> &SessionSummary {
        let title = derive_title(query);
        let session = SessionSummary {
            id: Uuid::new_v4(),
            title,
            query: query.to_string(),
            created_at: Utc::now(),
            is_active: true,
        };
        for s in &mut self.sessions {
            s.is_active = false;
        }
        self.sessions.insert(0, session);
        &self.sessions[0]
    }

    pub fn active(&self) -> Option<&SessionSummary> {
        self.sessions.iter().find(|s| s.is_active)
    }
}

fn derive_title(query: &str) -> String {
    let words: Vec<&str> = query.split_whitespace().take(5).collect();
    let title = words.join(" ");
    if title.len() > 40 {
        format!("{}...", &title[..37])
    } else if title.is_empty() {
        "Untitled Session".to_string()
    } else {
        let mut chars = title.chars();
        match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => title,
        }
    }
}

pub fn format_relative_time(now: DateTime<Utc>, then: DateTime<Utc>) -> String {
    let diff = now.signed_duration_since(then);
    let secs = diff.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 172800 {
        "Yesterday".to_string()
    } else {
        format!("{}d ago", secs / 86400)
    }
}
