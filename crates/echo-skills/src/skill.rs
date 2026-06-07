use serde_json::Value;

use crate::types::{SkillError, ToolDef};

/// A skill: a named bundle of callable tools (PLAN §6.4). Object-safe so the
/// registry can hold `Box<dyn Skill>`.
pub trait Skill {
    fn name(&self) -> &str;
    fn list_tools(&self) -> Vec<ToolDef>;
    fn call_tool(&self, tool: &str, args: &Value) -> Result<Value, SkillError>;
}
