//! Echo turn orchestrator: composes a Brain + MemoryStore into the voice-turn
//! lifecycle. Voice I/O (echo-voice traits) is driven by the shell around this.

mod agent;
mod context;
mod engine;

pub use agent::{AgentError, AgentLoop};
pub use context::build_system_context;
pub use engine::{TurnEngine, TurnError};
