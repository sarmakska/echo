use serde::{Deserialize, Serialize};

/// What a brain is good at. Mirrors PLAN.md §3.1 capability tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Code,
    Reason,
    ToolUse,
    Mcp,
    LongContext,
    Vision,
    ImageGen,
    WebSearch,
    WebGrounding,
    Workspace,
}

/// Remaining budget in the current quota window (PLAN.md §3.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quota {
    pub remaining: u32,
    pub limit: u32,
    pub window: String,
}

/// A single user request to a brain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt {
    pub text: String,
}

impl Prompt {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Context envelope injected as the system prompt (PLAN.md §4.3). Phase 1 carries
/// only the assembled system string; structured fields land with memory/router.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Context {
    pub system: String,
}

/// Normalized brain reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub brain_id: String,
    pub text: String,
}

/// Failures a brain can produce.
#[derive(Debug, thiserror::Error)]
pub enum BrainError {
    #[error("failed to spawn brain command `{command}`: {source}")]
    Spawn {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("brain exited with status {code:?}: {stderr}")]
    NonZeroExit { code: Option<i32>, stderr: String },
    #[error("could not parse brain output: {0}")]
    Parse(String),
}
