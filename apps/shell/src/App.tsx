import { useEffect, useState } from "react";
import { HudCard, type HudStatus } from "./components/HudCard";
import { CommandBar } from "./components/CommandBar";
import { SetupWizard, type EchoConfig } from "./components/SetupWizard";
import { speak } from "./lib/tts";
import { onEchoReply, onEchoTranscript, startListening } from "./lib/voice";

const SETUP_KEY = "echo.configured";

export function App() {
  const [configured, setConfigured] = useState(
    () => localStorage.getItem(SETUP_KEY) === "true",
  );
  const [reply, setReply] = useState("");
  const [status, setStatus] = useState<HudStatus>("idle");
  const [listening, setListening] = useState(false);

  // Reflect voice-worker events in the HUD when the voice build is running.
  useEffect(() => {
    if (!configured) return;
    const subs = [
      onEchoTranscript(() => setStatus("thinking")),
      onEchoReply((r) => {
        setReply(r);
        setStatus("idle");
      }),
    ];
    return () => {
      subs.forEach((p) => p.then((un) => un()).catch(() => {}));
    };
  }, [configured]);

  function onSetupComplete(config: EchoConfig) {
    // Persistence to ~/.echo/profile.yml via the shell is the production path;
    // the webview flag keeps the wizard from reappearing during local testing.
    localStorage.setItem(SETUP_KEY, "true");
    localStorage.setItem("echo.config", JSON.stringify(config));
    setConfigured(true);
  }

  async function toggleListening() {
    const err = await startListening();
    if (err) {
      setReply(err);
    } else {
      setListening(true);
      setStatus("listening");
    }
  }

  if (!configured) {
    return (
      <div className="flex h-screen w-screen items-start justify-end">
        <SetupWizard onComplete={onSetupComplete} />
      </div>
    );
  }

  return (
    <HudCard status={status}>
      <div className="flex flex-col gap-3">
        <p className="min-h-16 whitespace-pre-wrap text-slate-200">
          {reply || "Ready when you are."}
        </p>
        <CommandBar
          onReply={(r) => {
            setReply(r);
            setStatus("speaking");
            void speak(r).finally(() => setStatus("idle"));
          }}
          onBusyChange={(busy) => setStatus(busy ? "thinking" : "idle")}
        />
        <button
          type="button"
          onClick={toggleListening}
          disabled={listening}
          className="self-start rounded-lg border border-white/10 px-3 py-1 text-xs
                     text-slate-400 disabled:opacity-50"
        >
          {listening ? "Listening" : "Start listening"}
        </button>
      </div>
    </HudCard>
  );
}
