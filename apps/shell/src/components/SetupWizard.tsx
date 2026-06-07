import { useState } from "react";

export interface EchoConfig {
  brain: "claude" | "codex" | "gemini";
  skills: string[];
  wakeWord: string;
  voice: string;
}

const BRAINS: { id: EchoConfig["brain"]; label: string }[] = [
  { id: "claude", label: "Claude (Claude Code)" },
  { id: "codex", label: "Codex CLI" },
  { id: "gemini", label: "Gemini CLI" },
];

const SKILLS = [
  { id: "weather", label: "Weather" },
  { id: "web-search", label: "Web search" },
  { id: "files-local", label: "Local files" },
];

/// First-launch setup (day-one target, PLAN section 17): choose a brain, connect
/// the three Phase 1 skills, pick a wake word and a voice. Calls `onComplete`
/// with the assembled configuration. Persistence to profile.yml is handled by the
/// shell once the user finishes.
export function SetupWizard({ onComplete }: { onComplete: (config: EchoConfig) => void }) {
  const [step, setStep] = useState(0);
  const [brain, setBrain] = useState<EchoConfig["brain"]>("claude");
  const [skills, setSkills] = useState<string[]>(SKILLS.map((s) => s.id));
  const [wakeWord, setWakeWord] = useState("Echo");
  const [voice, setVoice] = useState("British female");

  function toggleSkill(id: string) {
    setSkills((cur) => (cur.includes(id) ? cur.filter((s) => s !== id) : [...cur, id]));
  }

  const steps = ["Brain", "Skills", "Wake word", "Voice"];
  const isLast = step === steps.length - 1;

  function next() {
    if (isLast) {
      onComplete({ brain, skills, wakeWord, voice });
    } else {
      setStep((s) => s + 1);
    }
  }

  return (
    <section
      aria-label="setup wizard"
      className="flex w-full flex-col gap-4 rounded-2xl border border-white/10
                 bg-slate-900/50 p-5 text-slate-100 shadow-2xl backdrop-blur-xl"
    >
      <header className="flex items-center justify-between text-sm">
        <span className="font-semibold text-[#22d3c5]">echo setup</span>
        <span className="text-slate-400">
          {step + 1} of {steps.length}: {steps[step]}
        </span>
      </header>

      {step === 0 && (
        <fieldset className="flex flex-col gap-2">
          <legend className="mb-1 text-slate-300">Choose your brain</legend>
          {BRAINS.map((b) => (
            <label key={b.id} className="flex items-center gap-2">
              <input
                type="radio"
                name="brain"
                value={b.id}
                checked={brain === b.id}
                onChange={() => setBrain(b.id)}
              />
              {b.label}
            </label>
          ))}
        </fieldset>
      )}

      {step === 1 && (
        <fieldset className="flex flex-col gap-2">
          <legend className="mb-1 text-slate-300">Connect skills</legend>
          {SKILLS.map((s) => (
            <label key={s.id} className="flex items-center gap-2">
              <input
                type="checkbox"
                value={s.id}
                checked={skills.includes(s.id)}
                onChange={() => toggleSkill(s.id)}
              />
              {s.label}
            </label>
          ))}
        </fieldset>
      )}

      {step === 2 && (
        <label className="flex flex-col gap-2 text-slate-300">
          Wake word
          <input
            aria-label="wake word"
            value={wakeWord}
            onChange={(e) => setWakeWord(e.target.value)}
            className="rounded-lg border border-white/10 bg-slate-800/60 px-3 py-1.5 text-slate-100"
          />
        </label>
      )}

      {step === 3 && (
        <label className="flex flex-col gap-2 text-slate-300">
          Voice
          <select
            aria-label="voice"
            value={voice}
            onChange={(e) => setVoice(e.target.value)}
            className="rounded-lg border border-white/10 bg-slate-800/60 px-3 py-1.5 text-slate-100"
          >
            <option>British female</option>
            <option>British male</option>
            <option>System default</option>
          </select>
        </label>
      )}

      <footer className="flex justify-between">
        <button
          type="button"
          onClick={() => setStep((s) => Math.max(0, s - 1))}
          disabled={step === 0}
          className="rounded-lg px-3 py-1.5 text-sm text-slate-400 disabled:opacity-40"
        >
          Back
        </button>
        <button
          type="button"
          onClick={next}
          className="rounded-lg bg-[#22d3c5]/20 px-4 py-1.5 text-sm font-medium text-[#22d3c5]"
        >
          {isLast ? "Finish" : "Next"}
        </button>
      </footer>
    </section>
  );
}
