use crate::brain::Brain;
use crate::parse::parse_stream_json;
use crate::runner::run_cli;
use crate::types::{BrainError, Capability, Context, Prompt, Response};

/// How to invoke the Claude CLI. Override `command` in tests to point at a fake.
#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub command: String,
    pub args: Vec<String>,
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            command: "claude".to_string(),
            args: vec![
                "--print".to_string(),
                "--output-format".to_string(),
                "stream-json".to_string(),
                "--verbose".to_string(),
            ],
        }
    }
}

const CLAUDE_CAPABILITIES: &[Capability] = &[
    Capability::Code,
    Capability::Reason,
    Capability::ToolUse,
    Capability::Mcp,
    Capability::LongContext,
    Capability::Vision,
];

/// Claude brain: spawns the configured CLI, pipes the prompt to stdin, parses
/// the stream-json stdout into a `Response`.
pub struct ClaudeBrain {
    config: BrainConfig,
}

impl ClaudeBrain {
    pub fn new(config: BrainConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(BrainConfig::default())
    }
}

impl Brain for ClaudeBrain {
    fn id(&self) -> &str {
        "claude"
    }

    fn capabilities(&self) -> &[Capability] {
        CLAUDE_CAPABILITIES
    }

    fn ask(&self, prompt: &Prompt, ctx: &Context) -> Result<Response, BrainError> {
        let stdout = run_cli(&self.config.command, &self.config.args, &ctx.system, &prompt.text)?;
        let text = parse_stream_json(&stdout)?;
        Ok(Response {
            brain_id: self.id().to_string(),
            text,
        })
    }
}
