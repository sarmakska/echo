use echo_memory::Turn;

/// Build the system-context string injected into the brain prompt (PLAN.md §4.3):
/// durable facts followed by the recent conversation. Pure and deterministic.
pub fn build_system_context(facts: &[String], recent: &[Turn]) -> String {
    let mut s = String::new();
    if !facts.is_empty() {
        s.push_str("# What you know about the user\n");
        for f in facts {
            s.push_str("- ");
            s.push_str(f);
            s.push('\n');
        }
    }
    if !recent.is_empty() {
        if !s.is_empty() {
            s.push('\n');
        }
        s.push_str("# Recent conversation\n");
        for t in recent {
            s.push_str(&t.role);
            s.push_str(": ");
            s.push_str(&t.text);
            s.push('\n');
        }
    }
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_when_nothing_known() {
        assert_eq!(build_system_context(&[], &[]), "");
    }

    #[test]
    fn includes_facts_and_recent_turns() {
        let facts = vec!["Lives in Hemel.".to_string()];
        let recent = vec![Turn::new("user", "hi", "t1"), Turn::new("echo", "hello", "t2")];
        let ctx = build_system_context(&facts, &recent);
        assert!(ctx.contains("Lives in Hemel."));
        assert!(ctx.contains("user: hi"));
        assert!(ctx.contains("echo: hello"));
    }
}
