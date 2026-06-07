import { invoke } from "@tauri-apps/api/core";

/// Speak text aloud via the shell's `speak` command. Never rejects; a failure
/// (e.g. no engine on this platform) is swallowed so it cannot break the UI.
export async function speak(text: string): Promise<void> {
  try {
    await invoke("speak", { text });
  } catch {
    // Speaking is best-effort; the reply is already shown in the HUD.
  }
}
