use std::io::{BufRead, Write};

use serde_json::{json, Value};

use crate::registry::SkillRegistry;

/// Exposes a `SkillRegistry` as an MCP server over stdio JSON-RPC (PLAN §6.4),
/// so Echo skills plug into any MCP-aware host (Claude Code, Codex, etc.) and so
/// Echo's own skill bus speaks one standard protocol. Implements the MCP methods
/// `initialize`, `tools/list`, and `tools/call`.
pub struct McpServer {
    registry: SkillRegistry,
}

impl McpServer {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    /// Handle one JSON-RPC 2.0 request object and produce the response object.
    pub fn handle_request(&self, req: &Value) -> Value {
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        match req.get("method").and_then(Value::as_str).unwrap_or("") {
            "initialize" => ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": { "name": "echo-skills", "version": "0.1.0" },
                    "capabilities": { "tools": {} }
                }),
            ),
            "tools/list" => {
                let tools: Vec<Value> = self
                    .registry
                    .all_tools()
                    .into_iter()
                    .map(|(_skill, t)| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": { "type": "object" }
                        })
                    })
                    .collect();
                ok(id, json!({ "tools": tools }))
            }
            "tools/call" => {
                let params = req.get("params");
                let name = params.and_then(|p| p.get("name")).and_then(Value::as_str);
                let args = params
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(Value::Null);
                match name {
                    None => err(id, -32602, "missing params.name"),
                    Some(tool) => match self.registry.call(tool, &args) {
                        Ok(result) => ok(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": result.to_string() }],
                                "isError": false
                            }),
                        ),
                        Err(e) => ok(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": e.to_string() }],
                                "isError": true
                            }),
                        ),
                    },
                }
            }
            other => err(id, -32601, &format!("method not found: {other}")),
        }
    }

    /// Run the newline-delimited JSON-RPC loop over the given reader/writer.
    /// (The binary entry point passes stdin/stdout; tests cover `handle_request`.)
    pub fn serve(&self, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let response = match serde_json::from_str::<Value>(&line) {
                Ok(req) => self.handle_request(&req),
                Err(e) => err(Value::Null, -32700, &format!("parse error: {e}")),
            };
            writeln!(output, "{response}")?;
            output.flush()?;
        }
        Ok(())
    }
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn err(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FakeHttp, WeatherSkill};

    fn server() -> McpServer {
        let mut reg = SkillRegistry::new();
        reg.register(Box::new(WeatherSkill::new(FakeHttp::ok(
            r#"{"current":{"temperature_2m":12.0,"weather_code":3}}"#,
        ))));
        McpServer::new(reg)
    }

    #[test]
    fn initialize_reports_server_info() {
        let r = server().handle_request(&json!({"jsonrpc":"2.0","id":1,"method":"initialize"}));
        assert_eq!(r["result"]["serverInfo"]["name"], "echo-skills");
        assert_eq!(r["id"], 1);
    }

    #[test]
    fn tools_list_includes_registered_tools() {
        let r = server().handle_request(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
        let tools = r["result"]["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "current_weather"));
    }

    #[test]
    fn tools_call_dispatches_and_returns_content() {
        let req = json!({
            "jsonrpc":"2.0","id":3,"method":"tools/call",
            "params":{"name":"current_weather","arguments":{"latitude":51.75,"longitude":-0.47}}
        });
        let r = server().handle_request(&req);
        assert_eq!(r["result"]["isError"], false);
        let text = r["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("temperature_c"));
    }

    #[test]
    fn tools_call_unknown_tool_is_tool_error() {
        let req = json!({
            "jsonrpc":"2.0","id":4,"method":"tools/call",
            "params":{"name":"nope","arguments":{}}
        });
        let r = server().handle_request(&req);
        assert_eq!(r["result"]["isError"], true);
    }

    #[test]
    fn unknown_method_is_jsonrpc_error() {
        let r = server().handle_request(&json!({"jsonrpc":"2.0","id":5,"method":"frobnicate"}));
        assert_eq!(r["error"]["code"], -32601);
    }
}
