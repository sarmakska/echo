use echo_brain::{Brain, Context, Prompt};
use echo_memory::{MemoryStore, Turn};

use crate::context::build_system_context;

#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error("brain error: {0}")]
    Brain(String),
    #[error("memory error: {0}")]
    Memory(String),
}

/// Composes a brain + memory store into the single-turn lifecycle (PLAN.md §4.3,
/// ARCHITECTURE request lifecycle). `day`/`ts` are injected (no clock here).
pub struct TurnEngine<B: Brain> {
    brain: B,
    memory: MemoryStore,
    recent_limit: usize,
}

impl<B: Brain> TurnEngine<B> {
    pub fn new(brain: B, memory: MemoryStore) -> Self {
        Self { brain, memory, recent_limit: 10 }
    }

    /// Handle one utterance: recall context, ask the brain, journal both turns,
    /// return the reply text.
    pub fn handle(&self, day: &str, ts: &str, utterance: &str) -> Result<String, TurnError> {
        let recent = self
            .memory
            .recent_turns(day, self.recent_limit)
            .map_err(|e| TurnError::Memory(e.to_string()))?;

        let slugs = self
            .memory
            .list_facts()
            .map_err(|e| TurnError::Memory(e.to_string()))?;
        let mut facts = Vec::new();
        for slug in slugs {
            if let Some(f) = self
                .memory
                .load_fact(&slug)
                .map_err(|e| TurnError::Memory(e.to_string()))?
            {
                facts.push(f.content);
            }
        }

        let system = build_system_context(&facts, &recent);
        let response = self
            .brain
            .ask(&Prompt::new(utterance), &Context { system })
            .map_err(|e| TurnError::Brain(e.to_string()))?;

        self.memory
            .append_turn(day, &Turn::new("user", utterance, ts))
            .map_err(|e| TurnError::Memory(e.to_string()))?;
        self.memory
            .append_turn(day, &Turn::new("echo", &response.text, ts))
            .map_err(|e| TurnError::Memory(e.to_string()))?;

        Ok(response.text)
    }
}
