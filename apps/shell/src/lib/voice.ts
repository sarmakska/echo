import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/// Start the always-listening voice worker. Returns null on success, or an error
/// message (e.g. when the build lacks the `voice` feature).
export async function startListening(): Promise<string | null> {
  try {
    await invoke("start_listening");
    return null;
  } catch (err) {
    return String(err);
  }
}

/// Subscribe to spoken replies emitted by the voice worker.
export function onEchoReply(cb: (reply: string) => void): Promise<UnlistenFn> {
  return listen<string>("echo://reply", (e) => cb(e.payload));
}

/// Subscribe to live transcripts emitted by the voice worker.
export function onEchoTranscript(cb: (transcript: string) => void): Promise<UnlistenFn> {
  return listen<string>("echo://transcript", (e) => cb(e.payload));
}
