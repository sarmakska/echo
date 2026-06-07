use serde_json::Value;

/// A request from a brain to invoke a tool (PLAN §6.4 / ARCHITECTURE step 7).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub tool: String,
    pub args: Value,
}

/// Detect a tool call embedded in a brain's reply text. Convention: a JSON
/// object somewhere in the text shaped `{"tool_call": {"name": "...", "args": {...}}}`.
/// The first such object wins. Returns None when the reply is a plain answer.
///
/// This is the brain-agnostic, in-process path. Brains with native MCP tool
/// loops (e.g. the Claude CLI wired to skill MCP servers) drive their own loop;
/// this covers brains without that and matches ARCHITECTURE step 7.
pub fn parse_tool_call(text: &str) -> Option<ToolCall> {
    // Scan for candidate JSON objects starting at each '{' and try to parse the
    // largest valid object from there. Cheap and robust enough for Phase 1.
    let bytes = text.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'{' {
            continue;
        }
        // Try progressively shorter slices is expensive; instead let serde's
        // streaming deserializer read one value from the offset.
        let mut de = serde_json::Deserializer::from_str(&text[i..]).into_iter::<Value>();
        if let Some(Ok(v)) = de.next() {
            if let Some(call) = v.get("tool_call") {
                let name = call.get("name").and_then(Value::as_str);
                if let Some(name) = name {
                    let args = call.get("args").cloned().unwrap_or(Value::Null);
                    return Some(ToolCall { tool: name.to_string(), args });
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_a_tool_call_object() {
        let text = r#"Let me check. {"tool_call":{"name":"current_weather","args":{"latitude":51.7,"longitude":-0.5}}}"#;
        let tc = parse_tool_call(text).unwrap();
        assert_eq!(tc.tool, "current_weather");
        assert_eq!(tc.args, json!({"latitude":51.7,"longitude":-0.5}));
    }

    #[test]
    fn plain_answer_has_no_tool_call() {
        assert!(parse_tool_call("It is sunny in Hemel today.").is_none());
    }

    #[test]
    fn tool_call_without_args_defaults_null() {
        let tc = parse_tool_call(r#"{"tool_call":{"name":"list_dir"}}"#).unwrap();
        assert_eq!(tc.tool, "list_dir");
        assert_eq!(tc.args, serde_json::Value::Null);
    }

    #[test]
    fn ignores_unrelated_json() {
        assert!(parse_tool_call(r#"here is data {"foo":1}"#).is_none());
    }
}
