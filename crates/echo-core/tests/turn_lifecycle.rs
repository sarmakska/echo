//! End-to-end single turn using a fake brain that echoes context awareness,
//! over a real temp-dir memory store.
use std::fs;

use echo_brain::{Brain, BrainError, Capability, Context, Prompt, Response};
use echo_core::TurnEngine;
use echo_memory::{Fact, MemoryStore};

/// Fake brain: replies with a fixed line and asserts it received some context.
struct FakeBrain;
impl Brain for FakeBrain {
    fn id(&self) -> &str {
        "fake"
    }
    fn capabilities(&self) -> &[Capability] {
        &[Capability::Reason]
    }
    fn ask(&self, prompt: &Prompt, ctx: &Context) -> Result<Response, BrainError> {
        // The prompt is the utterance; ctx.system carries recalled facts.
        let saw_fact = ctx.system.contains("Hemel");
        Ok(Response {
            brain_id: "fake".into(),
            text: format!("You said: {} (knew_location={})", prompt.text, saw_fact),
        })
    }
}

#[test]
fn handle_recalls_facts_asks_brain_and_journals() {
    let dir = std::env::temp_dir().join("echo-core-e2e");
    let _ = fs::remove_dir_all(&dir);
    let memory = MemoryStore::open(&dir).unwrap();
    memory
        .save_fact(&Fact { slug: "loc".into(), content: "Lives in Hemel.".into() })
        .unwrap();

    let engine = TurnEngine::new(FakeBrain, memory);
    let reply = engine.handle("2026/06/07", "09:00", "what is on today").unwrap();

    assert!(reply.contains("You said: what is on today"));
    assert!(reply.contains("knew_location=true"));

    // Both turns journaled.
    let memory2 = MemoryStore::open(&dir).unwrap();
    let turns = memory2.recent_turns("2026/06/07", 10).unwrap();
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].role, "user");
    assert_eq!(turns[1].role, "echo");
    assert_eq!(turns[1].text, reply);
}
