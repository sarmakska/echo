use std::fs;

use echo_skills::{FakeHttp, FilesSkill, SkillRegistry, WeatherSkill, WebSearchSkill};
use serde_json::json;

#[test]
fn registry_routes_to_three_skills() {
    let dir = std::env::temp_dir().join("echo-skills-e2e");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let mut reg = SkillRegistry::new();
    reg.register(Box::new(FilesSkill::new(&dir)));
    reg.register(Box::new(WeatherSkill::new(FakeHttp::ok(
        r#"{"current":{"temperature_2m":9.0,"weather_code":61}}"#,
    ))));
    reg.register(Box::new(WebSearchSkill::new(
        FakeHttp::ok(r#"{"results":[{"title":"Echo","url":"https://e.x"}]}"#),
        "https://searx.example/search",
    )));

    // 3 (files) + 1 (weather) + 1 (web-search) = 5 tools across the three skills.
    assert_eq!(reg.all_tools().len(), 5);

    reg.call("write_file", &json!({"path": "n.txt", "content": "ok"})).unwrap();
    assert_eq!(reg.call("read_file", &json!({"path": "n.txt"})).unwrap()["content"], "ok");
    assert_eq!(
        reg.call("current_weather", &json!({"latitude":51.75,"longitude":-0.47})).unwrap()
            ["weather_code"],
        61
    );
    assert_eq!(
        reg.call("web_search", &json!({"query":"echo"})).unwrap()["results"][0]["title"],
        "Echo"
    );
}
