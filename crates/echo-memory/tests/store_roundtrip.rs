//! A single session lifecycle over one temp root: write facts, journal turns,
//! save a digest, then read everything back the way a PreSession recall would.
use std::fs;

use echo_memory::{Fact, MemoryStore, Turn};

#[test]
fn full_session_lifecycle() {
    let dir = std::env::temp_dir().join("echo-mem-e2e-session");
    let _ = fs::remove_dir_all(&dir);
    let store = MemoryStore::open(&dir).unwrap();

    // Durable facts.
    store
        .save_fact(&Fact {
            slug: "project_echo".into(),
            content: "Echo is a local-first assistant.".into(),
        })
        .unwrap();

    // Journal a short conversation.
    store.append_turn("2026/06/07", &Turn::new("user", "what is on today", "09:00")).unwrap();
    store.append_turn("2026/06/07", &Turn::new("echo", "Standup at 9:30.", "09:01")).unwrap();

    // End-of-session digest.
    store
        .save_digest("session_2026-06-07-0901", "Discussed today's schedule.")
        .unwrap();

    // PreSession recall on next launch.
    assert_eq!(store.list_facts().unwrap(), vec!["project_echo"]);
    let recent = store.recent_turns("2026/06/07", 10).unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[1].text, "Standup at 9:30.");
    assert_eq!(
        store.latest_digest().unwrap().unwrap(),
        "Discussed today's schedule."
    );
}
