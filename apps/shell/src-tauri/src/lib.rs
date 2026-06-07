use tauri::{Manager, PhysicalPosition};

mod clock;

/// Compute the top-right position for a window, inset by `margin` px from the
/// top and right edges of a monitor. `mon` and `win` are (width, height) in px.
/// `x` is clamped to 0 so an over-wide window never lands off-screen left.
pub fn top_right_position(mon: (f64, f64), win: (f64, f64), margin: f64) -> (f64, f64) {
    let x = (mon.0 - win.0 - margin).max(0.0);
    let y = margin;
    (x, y)
}

/// Ask the default Claude brain a prompt directly (no memory). Returns the reply.
/// NOTE: requires the real `claude` CLI on PATH to succeed at runtime;
/// the brain logic itself is covered by `echo-brain`'s tests.
#[tauri::command]
fn ask_brain(prompt: String) -> Result<String, String> {
    use echo_brain::{Brain, ClaudeBrain, Context, Prompt};
    let brain = ClaudeBrain::with_defaults();
    brain
        .ask(&Prompt::new(prompt), &Context::default())
        .map(|r| r.text)
        .map_err(|e| e.to_string())
}

/// Run a full Echo turn through the memory-aware orchestrator: recall context
/// from `~/.echo/memory`, ask Claude, journal both turns, return the reply.
/// `day` is "YYYY/MM/DD" and `ts` an ISO timestamp, supplied by the caller.
/// NOTE: needs the real `claude` CLI at runtime; the engine is covered by
/// `echo-core`'s tests.
#[tauri::command]
fn echo_turn(
    app: tauri::AppHandle,
    prompt: String,
    day: String,
    ts: String,
) -> Result<String, String> {
    use echo_brain::ClaudeBrain;
    use echo_core::TurnEngine;
    use echo_memory::MemoryStore;

    let home = app.path().home_dir().map_err(|e| e.to_string())?;
    let root = home.join(".echo").join("memory");
    let memory = MemoryStore::open(&root).map_err(|e| e.to_string())?;
    let engine = TurnEngine::new(ClaudeBrain::with_defaults(), memory);
    engine.handle(&day, &ts, &prompt).map_err(|e| e.to_string())
}

/// Speak text aloud. Uses the macOS `say` engine for now; other platforms get
/// their TTS adapter wired in a later step (Piper is the cross-platform default).
#[tauri::command]
fn speak(text: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use echo_voice::{SayTts, Tts};
        return SayTts::new(None).speak(&text).map_err(|e| e.to_string());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        Err("speak: only the macOS engine is wired so far".to_string())
    }
}

/// Spawn the live voice worker (cpal mic -> VAD wake -> whisper.cpp STT ->
/// TurnEngine -> macOS TTS), emitting `echo://transcript` and `echo://reply`
/// events to the HUD. Built only with `--features voice` on macOS; needs the
/// whisper.cpp binary plus `~/.echo/models/ggml-small.en.bin` and a mic at runtime.
#[cfg(all(feature = "voice", target_os = "macos"))]
fn spawn_voice_worker(app: tauri::AppHandle) {
    use echo_brain::ClaudeBrain;
    use echo_core::TurnEngine;
    use echo_memory::MemoryStore;
    use echo_voice::{CpalMic, EnergyVadWakeWord, SayTts, VoiceLoop, WhisperCliStt};
    use tauri::Emitter;

    std::thread::spawn(move || {
        let Ok(home) = app.path().home_dir() else {
            return;
        };
        let model = home.join(".echo/models/ggml-small.en.bin");
        let memory = match MemoryStore::open(home.join(".echo/memory")) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("voice worker: memory open failed: {e}");
                return;
            }
        };
        let engine = TurnEngine::new(ClaudeBrain::with_defaults(), memory);

        let mut voice = VoiceLoop::new(
            EnergyVadWakeWord::default(),
            CpalMic::new(4000),
            WhisperCliStt::new("whisper-cli", model.to_string_lossy().to_string()),
            SayTts::new(None),
        );

        let mut handler = |transcript: &str| -> String {
            let _ = app.emit("echo://transcript", transcript.to_string());
            let reply = engine
                .handle(&clock::today_utc(), &clock::now_iso(), transcript)
                .unwrap_or_else(|e| format!("Echo hit an error: {e}"));
            let _ = app.emit("echo://reply", reply.clone());
            reply
        };
        let _ = voice.run(&mut handler);
    });
}

/// Start the always-listening voice loop. No-op error unless built with the
/// `voice` feature on a supported platform.
#[tauri::command]
fn start_listening(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(all(feature = "voice", target_os = "macos"))]
    {
        spawn_voice_worker(app);
        Ok(())
    }
    #[cfg(not(all(feature = "voice", target_os = "macos")))]
    {
        let _ = app;
        Err("voice loop not built: rebuild with `--features voice` on macOS".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ask_brain, echo_turn, speak, start_listening])
        .setup(|app| {
            if let Some(win) = app.get_webview_window("hud") {
                if let (Ok(Some(monitor)), Ok(size)) =
                    (win.current_monitor(), win.outer_size())
                {
                    let m = monitor.size();
                    let (x, y) = top_right_position(
                        (m.width as f64, m.height as f64),
                        (size.width as f64, size.height as f64),
                        24.0,
                    );
                    let _ = win.set_position(PhysicalPosition::new(x, y));
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Echo");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_window_at_top_right_with_margin() {
        let (x, y) = top_right_position((1920.0, 1080.0), (380.0, 540.0), 24.0);
        assert_eq!(x, 1920.0 - 380.0 - 24.0);
        assert_eq!(y, 24.0);
    }

    #[test]
    fn clamps_x_to_zero_when_window_wider_than_monitor() {
        let (x, _) = top_right_position((300.0, 1080.0), (380.0, 540.0), 24.0);
        assert_eq!(x, 0.0);
    }
}
