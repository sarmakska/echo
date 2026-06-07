//! Echo brain workers. Phase 1 ships the Claude subprocess brain; Phase 3 adds
//! the Codex + Gemini brains and the capability/quota router.

mod brain;
mod claude;
mod codex;
mod gemini;
mod parse;
mod router;
mod runner;
mod types;

pub use brain::Brain;
pub use claude::{BrainConfig, ClaudeBrain};
pub use codex::CodexBrain;
pub use gemini::GeminiBrain;
pub use parse::parse_stream_json;
pub use router::{capability_match, pick_brain, score, BrainCandidate, BrainScoreInput, RouteRequest};
pub use types::{BrainError, Capability, Context, Prompt, Quota, Response};
