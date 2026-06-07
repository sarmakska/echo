use crate::brain::Brain;
use crate::parse::parse_stream_json;
use crate::runner::run_cli;
use crate::types::{BrainError, Capability, Context, Prompt, Response};

/// Google Gemini CLI brain (PLAN §3). Subprocess wrapper around `gemini`.
///
/// NOTE: the exact `gemini` output schema is provisional here — it reuses the
/// result/assistant-text shape (`parse_stream_json`) and must be validated
/// against the real CLI when Gemini is installed.
pub struct GeminiBrain {
    command: String,
    args: Vec<String>,
}

const GEMINI_CAPABILITIES: &[Capability] = &[
    Capability::Reason,
    Capability::LongContext,
    Capability::WebGrounding,
    Capability::Workspace,
    Capability::Vision,
];

impl Default for GeminiBrain {
    fn default() -> Self {
        Self {
            command: "gemini".to_string(),
            args: vec!["--json".to_string()],
        }
    }
}

impl GeminiBrain {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self { command: command.into(), args }
    }
}

impl Brain for GeminiBrain {
    fn id(&self) -> &str {
        "gemini"
    }
    fn capabilities(&self) -> &[Capability] {
        GEMINI_CAPABILITIES
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
    fn advertises_gemini_identity_and_caps() {
        let b = GeminiBrain::default();
        assert_eq!(b.id(), "gemini");
        assert!(b.capabilities().contains(&Capability::WebGrounding));
    }
}
