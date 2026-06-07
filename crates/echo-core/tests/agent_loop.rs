//! The agentic loop: a fake brain asks for a tool on the first turn, then uses
//! the tool result to answer on the second turn. A real weather skill (with a
//! fake HTTP client) services the call.
use std::cell::Cell;

use echo_brain::{Brain, BrainError, Capability, Context, Prompt, Response};
use echo_core::AgentLoop;
use echo_skills::{FakeHttp, SkillRegistry, WeatherSkill};

/// Brain that emits a tool call until it sees a tool result in the conversation,
/// then produces a final answer using it.
struct ToolThenAnswerBrain {
    calls: Cell<u32>,
}
impl Brain for ToolThenAnswerBrain {
    fn id(&self) -> &str {
        "fake"
    }
    fn capabilities(&self) -> &[Capability] {
        &[Capability::ToolUse]
    }
    fn ask(&self, prompt: &Prompt, _ctx: &Context) -> Result<Response, BrainError> {
        self.calls.set(self.calls.get() + 1);
        let text = if prompt.text.contains("[tool current_weather result]") {
            "It is 9°C and rainy in Hemel.".to_string()
        } else {
            r#"{"tool_call":{"name":"current_weather","args":{"latitude":51.75,"longitude":-0.47}}}"#
                .to_string()
        };
        Ok(Response { brain_id: "fake".into(), text })
    }
}

#[test]
fn loop_dispatches_a_tool_then_answers() {
    let brain = ToolThenAnswerBrain { calls: Cell::new(0) };
    let mut skills = SkillRegistry::new();
    skills.register(Box::new(WeatherSkill::new(FakeHttp::ok(
        r#"{"current":{"temperature_2m":9.0,"weather_code":61}}"#,
    ))));

    let agent = AgentLoop::new(&brain, &skills);
    let answer = agent.run("", "what is the weather in Hemel").unwrap();

    assert_eq!(answer, "It is 9°C and rainy in Hemel.");
    assert_eq!(brain.calls.get(), 2); // tool turn + answer turn
}

#[test]
fn loop_returns_plain_answer_without_tools() {
    let brain = ToolThenAnswerBrain { calls: Cell::new(0) };
    // Pre-seed so the brain answers immediately (conversation already has a result).
    let skills = SkillRegistry::new();
    let agent = AgentLoop::new(&brain, &skills);
    let answer = agent
        .run("", "context with [tool current_weather result]: {} already")
        .unwrap();
    assert!(answer.contains("9°C"));
    assert_eq!(brain.calls.get(), 1);
}
