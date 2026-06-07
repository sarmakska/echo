use serde_json::{json, Value};

use crate::http::HttpGet;
use crate::skill::Skill;
use crate::types::{SkillError, ToolDef};

/// Parse a SearXNG-shaped JSON results body into a compact top-N list.
pub fn parse_search_results(body: &str, limit: usize) -> Result<Value, SkillError> {
    let v: Value = serde_json::from_str(body).map_err(|e| SkillError::Parse(e.to_string()))?;
    let results = v
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| SkillError::Parse("no `results` array".into()))?;
    let items: Vec<Value> = results
        .iter()
        .take(limit)
        .map(|r| {
            json!({
                "title": r.get("title").and_then(Value::as_str).unwrap_or(""),
                "url": r.get("url").and_then(Value::as_str).unwrap_or(""),
            })
        })
        .collect();
    Ok(json!({ "results": items }))
}

/// Web search over a SearXNG-compatible JSON endpoint. Endpoint is configurable.
pub struct WebSearchSkill<H: HttpGet> {
    http: H,
    endpoint: String,
}

impl<H: HttpGet> WebSearchSkill<H> {
    /// `endpoint` is the base, e.g. "https://searx.example.org/search".
    pub fn new(http: H, endpoint: impl Into<String>) -> Self {
        Self { http, endpoint: endpoint.into() }
    }
}

impl<H: HttpGet> Skill for WebSearchSkill<H> {
    fn name(&self) -> &str {
        "web-search"
    }

    fn list_tools(&self) -> Vec<ToolDef> {
        vec![ToolDef::new("web_search", "Search the web; returns top result titles + URLs")]
    }

    fn call_tool(&self, tool: &str, args: &Value) -> Result<Value, SkillError> {
        if tool != "web_search" {
            return Err(SkillError::UnknownTool(tool.to_string()));
        }
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .ok_or_else(|| SkillError::BadArgs("missing query".into()))?;
        let url = format!("{}?q={}&format=json", self.endpoint, urlencode(query));
        let body = self.http.get(&url)?;
        parse_search_results(&body, 5)
    }
}

/// Minimal percent-encoding for query strings (space + a few reserved chars).
fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::FakeHttp;

    const SAMPLE: &str = r#"{"results":[
        {"title":"Rust","url":"https://rust-lang.org","content":"x"},
        {"title":"Tauri","url":"https://tauri.app","content":"y"}
    ]}"#;

    #[test]
    fn parses_top_results() {
        let out = parse_search_results(SAMPLE, 5).unwrap();
        assert_eq!(out["results"][0]["title"], "Rust");
        assert_eq!(out["results"][1]["url"], "https://tauri.app");
    }

    #[test]
    fn respects_limit() {
        let out = parse_search_results(SAMPLE, 1).unwrap();
        assert_eq!(out["results"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn parse_errors_without_results() {
        assert!(matches!(parse_search_results("{}", 5).unwrap_err(), SkillError::Parse(_)));
    }

    #[test]
    fn call_tool_searches_and_parses() {
        let skill = WebSearchSkill::new(FakeHttp::ok(SAMPLE), "https://searx.example/search");
        let out = skill.call_tool("web_search", &json!({"query": "rust lang"})).unwrap();
        assert_eq!(out["results"][0]["title"], "Rust");
    }

    #[test]
    fn urlencode_escapes_spaces() {
        assert_eq!(urlencode("a b"), "a%20b");
    }
}
