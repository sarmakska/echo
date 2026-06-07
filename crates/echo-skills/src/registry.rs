use serde_json::Value;

use crate::skill::Skill;
use crate::types::{SkillError, ToolDef};

/// Holds installed skills and routes a tool call to whichever skill owns it.
#[derive(Default)]
pub struct SkillRegistry {
    skills: Vec<Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        self.skills.push(skill);
    }

    /// All tools across all skills, paired with their owning skill name.
    pub fn all_tools(&self) -> Vec<(String, ToolDef)> {
        let mut out = Vec::new();
        for s in &self.skills {
            for t in s.list_tools() {
                out.push((s.name().to_string(), t));
            }
        }
        out
    }

    /// Dispatch a tool call to the first skill exposing `tool`.
    pub fn call(&self, tool: &str, args: &Value) -> Result<Value, SkillError> {
        for s in &self.skills {
            if s.list_tools().iter().any(|t| t.name == tool) {
                return s.call_tool(tool, args);
            }
        }
        Err(SkillError::UnknownTool(tool.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ToolDef;

    struct PingSkill;
    impl Skill for PingSkill {
        fn name(&self) -> &str {
            "ping"
        }
        fn list_tools(&self) -> Vec<ToolDef> {
            vec![ToolDef::new("ping", "returns pong")]
        }
        fn call_tool(&self, tool: &str, _args: &Value) -> Result<Value, SkillError> {
            match tool {
                "ping" => Ok(Value::String("pong".into())),
                other => Err(SkillError::UnknownTool(other.into())),
            }
        }
    }

    #[test]
    fn dispatches_to_owning_skill() {
        let mut reg = SkillRegistry::new();
        reg.register(Box::new(PingSkill));
        assert_eq!(reg.call("ping", &Value::Null).unwrap(), Value::String("pong".into()));
    }

    #[test]
    fn unknown_tool_errors() {
        let reg = SkillRegistry::new();
        assert!(matches!(reg.call("nope", &Value::Null).unwrap_err(), SkillError::UnknownTool(_)));
    }

    #[test]
    fn all_tools_lists_with_skill_name() {
        let mut reg = SkillRegistry::new();
        reg.register(Box::new(PingSkill));
        let tools = reg.all_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "ping");
        assert_eq!(tools[0].1.name, "ping");
    }
}
