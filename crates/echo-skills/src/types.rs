use serde::{Deserialize, Serialize};

/// Describes one callable tool a skill exposes (PLAN §6.1 / §6.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
}

impl ToolDef {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { name: name.into(), description: description.into() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("bad arguments: {0}")]
    BadArgs(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("parse error: {0}")]
    Parse(String),
}
