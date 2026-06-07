import { invoke } from "@tauri-apps/api/core";

/// Ask the Rust `ask_brain` command (which drives the Claude CLI) for a reply.
/// Never rejects: a CLI/transport failure is returned as user-facing text so the
/// HUD can simply display it.
export async function askBrain(prompt: string): Promise<string> {
  try {
    return await invoke<string>("ask_brain", { prompt });
  } catch (err) {
    return `Echo couldn't reach the brain: ${String(err)}`;
  }
}
