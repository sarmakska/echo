import { useState } from "react";
import { echoTurn } from "../lib/turn";

/// Keyboard input bar (PLAN §5.5): type a prompt, Echo replies via the brain.
/// Calls `onReply` with the reply text, or an error string on failure.
/// `onBusyChange` lets the parent reflect thinking/idle status in the HUD.
export function CommandBar({
  onReply,
  onBusyChange,
}: {
  onReply?: (reply: string) => void;
  onBusyChange?: (busy: boolean) => void;
}) {
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);

  function submit(e: React.FormEvent) {
    e.preventDefault();
    const prompt = text.trim();
    if (!prompt || busy) return;
    setBusy(true);
    onBusyChange?.(true);
    // echoTurn never rejects — failures come back as display text.
    echoTurn(prompt).then((reply) => {
      onReply?.(reply);
      setText("");
      setBusy(false);
      onBusyChange?.(false);
    });
  }

  return (
    <form onSubmit={submit} className="flex gap-2">
      <input
        aria-label="command"
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder="Ask Echo…"
        disabled={busy}
        className="flex-1 rounded-lg border border-white/10 bg-slate-800/60 px-3 py-1.5
                   text-sm text-slate-100 placeholder:text-slate-500
                   focus:border-[#22d3c5] focus:outline-none"
      />
      <button
        type="submit"
        disabled={busy}
        className="rounded-lg bg-[#22d3c5]/20 px-3 py-1.5 text-sm font-medium
                   text-[#22d3c5] disabled:opacity-50"
      >
        {busy ? "…" : "Ask"}
      </button>
    </form>
  );
}
