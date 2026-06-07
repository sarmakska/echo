use std::io::Write;
use std::process::{Command, Stdio};

use crate::types::BrainError;

/// Spawn a subscription-backed CLI, pipe `prompt` to stdin (optionally appending
/// a system prompt), and return captured stdout. Shared by every CLI brain so
/// the spawn/stdin/exit-handling lives in one tested place.
pub(crate) fn run_cli(
    command: &str,
    args: &[String],
    system: &str,
    prompt: &str,
) -> Result<String, BrainError> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if !system.is_empty() {
        cmd.arg("--append-system-prompt").arg(system);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|source| BrainError::Spawn {
        command: command.to_string(),
        source,
    })?;

    {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| BrainError::Parse("child stdin unavailable".to_string()))?;
        let mut stdin = stdin;
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|source| BrainError::Spawn {
                command: command.to_string(),
                source,
            })?;
    } // stdin dropped here → EOF

    let output = child.wait_with_output().map_err(|source| BrainError::Spawn {
        command: command.to_string(),
        source,
    })?;

    if !output.status.success() {
        return Err(BrainError::NonZeroExit {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
