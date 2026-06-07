import { invoke } from "@tauri-apps/api/core";

function pad(n: number): string {
  return String(n).padStart(2, "0");
}

/// Format a Date as the memory store's "YYYY/MM/DD" episode key.
export function episodeDay(d: Date): string {
  return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())}`;
}

/// Run a full memory-aware Echo turn via the Rust `echo_turn` command.
/// Never rejects: failures come back as user-facing display text.
export async function echoTurn(prompt: string, now: Date = new Date()): Promise<string> {
  try {
    return await invoke<string>("echo_turn", {
      prompt,
      day: episodeDay(now),
      ts: now.toISOString(),
    });
  } catch (err) {
    return `Echo couldn't reach the brain: ${String(err)}`;
  }
}
