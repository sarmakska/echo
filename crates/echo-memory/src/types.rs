use serde::{Deserialize, Serialize};

/// One conversational turn, journaled to a daily episode file (PLAN.md §7.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    /// "user" or "echo".
    pub role: String,
    /// What was said.
    pub text: String,
    /// ISO-8601 timestamp, supplied by the caller (no clock in this crate).
    pub ts: String,
}

impl Turn {
    pub fn new(role: impl Into<String>, text: impl Into<String>, ts: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            text: text.into(),
            ts: ts.into(),
        }
    }
}

/// A durable fact, one Markdown file per fact (PLAN.md §7.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// Filename stem, e.g. "sarma_lives_hemel".
    pub slug: String,
    /// Markdown body.
    pub content: String,
}
