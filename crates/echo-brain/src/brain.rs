use crate::types::{BrainError, Capability, Context, Prompt, Response};

/// A subscription-backed brain. Phase 1 is the synchronous `ask` subset of
/// PLAN.md §3.3; async streaming + live quota arrive with the Phase 3 router.
pub trait Brain {
    /// Stable identifier, e.g. "claude".
    fn id(&self) -> &str;

    /// What this brain is good at (PLAN.md §3.1).
    fn capabilities(&self) -> &[Capability];

    /// Ask a single prompt with context, return normalized text.
    fn ask(&self, prompt: &Prompt, ctx: &Context) -> Result<Response, BrainError>;
}
