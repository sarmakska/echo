use crate::types::BrainError;
use serde_json::Value;

/// Parse Claude CLI `--output-format stream-json` output (one JSON object per line)
/// into the final assistant text.
///
/// Resolution order:
/// 1. The last line with `type == "result"` and a string `result` field wins.
/// 2. Otherwise, concatenate `text` from every `assistant` message content block.
/// 3. Otherwise, error.
///
/// Blank lines are ignored. A non-blank line that is not valid JSON is an error.
pub fn parse_stream_json(stdout: &str) -> Result<String, BrainError> {
    let mut result_text: Option<String> = None;
    let mut assistant_text = String::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line)
            .map_err(|e| BrainError::Parse(format!("invalid json line: {e}")))?;

        match v.get("type").and_then(Value::as_str) {
            Some("result") => {
                if let Some(r) = v.get("result").and_then(Value::as_str) {
                    result_text = Some(r.to_string());
                }
            }
            Some("assistant") => {
                if let Some(blocks) = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(Value::as_array)
                {
                    for b in blocks {
                        if b.get("type").and_then(Value::as_str) == Some("text") {
                            if let Some(t) = b.get("text").and_then(Value::as_str) {
                                assistant_text.push_str(t);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(r) = result_text {
        return Ok(r.trim().to_string());
    }
    if !assistant_text.is_empty() {
        return Ok(assistant_text.trim().to_string());
    }
    Err(BrainError::Parse("no assistant text in stream".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_result_line() {
        let s = r#"{"type":"system","subtype":"init"}
{"type":"assistant","message":{"content":[{"type":"text","text":"partial"}]}}
{"type":"result","subtype":"success","result":"Sunny in Hemel."}"#;
        assert_eq!(parse_stream_json(s).unwrap(), "Sunny in Hemel.");
    }

    #[test]
    fn accumulates_assistant_text_without_result_line() {
        let s = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hello "}]}}
{"type":"assistant","message":{"content":[{"type":"text","text":"world"}]}}"#;
        assert_eq!(parse_stream_json(s).unwrap(), "Hello world");
    }

    #[test]
    fn ignores_blank_lines() {
        let s = "\n{\"type\":\"result\",\"result\":\"ok\"}\n\n";
        assert_eq!(parse_stream_json(s).unwrap(), "ok");
    }

    #[test]
    fn errors_on_invalid_json_line() {
        let err = parse_stream_json("not json").unwrap_err();
        assert!(matches!(err, BrainError::Parse(_)));
    }

    #[test]
    fn errors_when_no_text_present() {
        let s = r#"{"type":"system","subtype":"init"}"#;
        assert!(matches!(parse_stream_json(s).unwrap_err(), BrainError::Parse(_)));
    }
}
