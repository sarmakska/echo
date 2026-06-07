use serde_json::{json, Value};

use crate::http::HttpGet;
use crate::skill::Skill;
use crate::types::{SkillError, ToolDef};

/// Parse an Open-Meteo `current` forecast body into a compact summary.
pub fn parse_open_meteo(body: &str) -> Result<Value, SkillError> {
    let v: Value = serde_json::from_str(body).map_err(|e| SkillError::Parse(e.to_string()))?;
    let cur = v.get("current").ok_or_else(|| SkillError::Parse("no `current` field".into()))?;
    let temp = cur
        .get("temperature_2m")
        .and_then(Value::as_f64)
        .ok_or_else(|| SkillError::Parse("no temperature_2m".into()))?;
    let code = cur.get("weather_code").and_then(Value::as_i64).unwrap_or(-1);
    Ok(json!({ "temperature_c": temp, "weather_code": code }))
}

/// Weather skill backed by Open-Meteo (no API key). Generic over the HTTP client.
pub struct WeatherSkill<H: HttpGet> {
    http: H,
}

impl<H: HttpGet> WeatherSkill<H> {
    pub fn new(http: H) -> Self {
        Self { http }
    }
}

impl<H: HttpGet> Skill for WeatherSkill<H> {
    fn name(&self) -> &str {
        "weather"
    }

    fn list_tools(&self) -> Vec<ToolDef> {
        vec![ToolDef::new("current_weather", "Current weather for a lat/lon via Open-Meteo")]
    }

    fn call_tool(&self, tool: &str, args: &Value) -> Result<Value, SkillError> {
        if tool != "current_weather" {
            return Err(SkillError::UnknownTool(tool.to_string()));
        }
        let lat = args
            .get("latitude")
            .and_then(Value::as_f64)
            .ok_or_else(|| SkillError::BadArgs("missing latitude".into()))?;
        let lon = args
            .get("longitude")
            .and_then(Value::as_f64)
            .ok_or_else(|| SkillError::BadArgs("missing longitude".into()))?;
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={lat}&longitude={lon}&current=temperature_2m,weather_code"
        );
        let body = self.http.get(&url)?;
        parse_open_meteo(&body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::FakeHttp;

    const SAMPLE: &str = r#"{"current":{"temperature_2m":12.4,"weather_code":3}}"#;

    #[test]
    fn parses_temperature_and_code() {
        let out = parse_open_meteo(SAMPLE).unwrap();
        assert_eq!(out["temperature_c"], 12.4);
        assert_eq!(out["weather_code"], 3);
    }

    #[test]
    fn parse_errors_without_current() {
        assert!(matches!(parse_open_meteo("{}").unwrap_err(), SkillError::Parse(_)));
    }

    #[test]
    fn call_tool_uses_http_and_parses() {
        let skill = WeatherSkill::new(FakeHttp::ok(SAMPLE));
        let out = skill
            .call_tool("current_weather", &json!({"latitude": 51.75, "longitude": -0.47}))
            .unwrap();
        assert_eq!(out["temperature_c"], 12.4);
    }

    #[test]
    fn call_tool_missing_coords_errors() {
        let skill = WeatherSkill::new(FakeHttp::ok(SAMPLE));
        assert!(matches!(
            skill.call_tool("current_weather", &json!({})).unwrap_err(),
            SkillError::BadArgs(_)
        ));
    }
}
