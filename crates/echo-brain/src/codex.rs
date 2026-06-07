use crate::brain::Brain;
use crate::parse::parse_stream_json;
use crate::runner::run_cli;
use crate::types::{BrainError, Capability, Context, Prompt, Response};

/// OpenAI Codex CLI brain (PLAN §3). Subprocess wrapper around `codex`.
///
/// NOTE: the exact `codex` stream event schema is provisional here — it reuses
/// the result/assistant-text shape (`parse_stream_json`) and must be validated
/// against the real CLI when Codex is installed.
pub struct CodexBrain {
    command: String,
    args: Vec<String>,
}

const CODEX_CAPABILITIES: &[Capability] = &[
    Capability::Code,
    Capability::Reason,
    Capability::Vision,
    Capability::ImageGen,
    Capability::WebSearch,
];

impl Default for CodexBrain {
    fn default() -> Self {
        Self {
            command: "codex".to_string(),
            args: vec!["exec".to_string(), "--json".to_string()],
        }
    }
}

impl CodexBrain {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self { command: command.into(), args }
    }
}

impl Brain for CodexBrain {
    fn id(&self) -> &str {
        "codex"
    }
    fn capabilities(&self) -> &[Capability] {
        CODEX_CAPABILITIES
    }
    fn ask(&self, prompt: &Prompt, ctx: &Context) -> Result<Response, BrainError> {
        let stdout = run_cli(&self.command, &self.args, &ctx.system, &prompt.text)?;
        Ok(Response {
            brain_id: self.id().to_string(),
            text: parse_stream_json(&stdout)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertises_codex_identity_and_caps() {
        let b = CodexBrain::default();
        assert_eq!(b.id(), "codex");
        assert!(b.capabilities().contains(&Capability::ImageGen));
    }
}
