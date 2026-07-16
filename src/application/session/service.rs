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

    pub fn create(&mut self, query: &str) -> Uuid {
        let id = Uuid::new_v4();
        self.insert_front(SessionSummary {
            id,
            title: derive_title(query),
            query: query.to_string(),
            created_at: Utc::now(),
            is_active: true,
        });
        id
    }

    pub fn insert_front(&mut self, mut session: SessionSummary) {
        for s in &mut self.sessions {
            s.is_active = false;
        }
        session.is_active = true;
        self.sessions.retain(|s| s.id != session.id);
        self.sessions.insert(0, session);
    }

    pub fn replace_all(&mut self, mut sessions: Vec<SessionSummary>) {
        for s in &mut sessions {
            s.is_active = false;
        }
        if let Some(first) = sessions.first_mut() {
            first.is_active = true;
        }
        self.sessions = sessions;
    }

    pub fn active(&self) -> Option<&SessionSummary> {
        self.sessions.iter().find(|s| s.is_active)
    }

    pub fn get(&self, index: usize) -> Option<&SessionSummary> {
        self.sessions.get(index)
    }

    pub fn select(&mut self, index: usize) {
        if index >= self.sessions.len() {
            return;
        }
        for s in &mut self.sessions {
            s.is_active = false;
        }
        self.sessions[index].is_active = true;
    }

    pub fn remove(&mut self, index: usize) -> Option<Uuid> {
        if index >= self.sessions.len() {
            return None;
        }
        let removed = self.sessions.remove(index);
        let was_active = removed.is_active;
        let id = removed.id;
        if was_active && let Some(first) = self.sessions.first_mut() {
            first.is_active = true;
        }
        Some(id)
    }
}

impl From<crate::domain::traits::session_store::SessionSummary> for SessionSummary {
    fn from(s: crate::domain::traits::session_store::SessionSummary) -> Self {
        Self {
            id: s.id,
            title: if s.title.is_empty() {
                derive_title(&s.query)
            } else {
                derive_title(&s.title)
            },
            query: s.query,
            created_at: s.created_at,
            is_active: s.is_active,
        }
    }
}

pub fn derive_title(query: &str) -> String {
    let words: Vec<&str> = query.split_whitespace().take(5).collect();
    let title = words.join(" ");
    if title.is_empty() {
        return "Untitled Session".to_string();
    }
    if title.chars().count() > 40 {
        let truncated: String = title.chars().take(37).collect();
        return format!("{truncated}...");
    }
    let mut chars = title.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => title,
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
