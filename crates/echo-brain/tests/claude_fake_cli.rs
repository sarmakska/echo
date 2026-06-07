//! Proves the full spawn → stdin → stdout → parse pipeline without the real
//! `claude` binary, by pointing BrainConfig at a generated fake CLI script.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use echo_brain::{Brain, BrainConfig, ClaudeBrain, Context, Prompt};

fn write_fake_cli(dir: &PathBuf, name: &str, body: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path.to_string_lossy().to_string()
}

fn temp_subdir(tag: &str) -> PathBuf {
    // Unique-enough per test name; std-only (no tempfile dep).
    let dir = std::env::temp_dir().join(format!("echo-brain-test-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn ask_returns_parsed_result_text() {
    let dir = temp_subdir("ok");
    let script = r#"#!/bin/sh
cat > /dev/null   # consume the prompt on stdin
cat <<'JSON'
{"type":"system","subtype":"init"}
{"type":"assistant","message":{"content":[{"type":"text","text":"ignored partial"}]}}
{"type":"result","subtype":"success","result":"It is sunny in Hemel today."}
JSON
"#;
    let cmd = write_fake_cli(&dir, "fake-claude.sh", script);

    let brain = ClaudeBrain::new(BrainConfig { command: cmd, args: vec![] });
    let resp = brain
        .ask(&Prompt::new("what is the weather"), &Context::default())
        .unwrap();

    assert_eq!(resp.brain_id, "claude");
    assert_eq!(resp.text, "It is sunny in Hemel today.");
}

#[test]
fn ask_surfaces_non_zero_exit() {
    let dir = temp_subdir("fail");
    let script = "#!/bin/sh\necho 'boom' 1>&2\nexit 3\n";
    let cmd = write_fake_cli(&dir, "fake-claude.sh", script);

    let brain = ClaudeBrain::new(BrainConfig { command: cmd, args: vec![] });
    let err = brain
        .ask(&Prompt::new("hi"), &Context::default())
        .unwrap_err();

    match err {
        echo_brain::BrainError::NonZeroExit { code, stderr } => {
            assert_eq!(code, Some(3));
            assert!(stderr.contains("boom"));
        }
        other => panic!("expected NonZeroExit, got {other:?}"),
    }
}

#[test]
fn ask_errors_when_command_missing() {
    let brain = ClaudeBrain::new(BrainConfig {
        command: "/nonexistent/echo-brain-no-such-binary".to_string(),
        args: vec![],
    });
    let err = brain
        .ask(&Prompt::new("hi"), &Context::default())
        .unwrap_err();
    assert!(matches!(err, echo_brain::BrainError::Spawn { .. }));
}
