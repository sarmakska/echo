import type { ReactNode } from "react";

export type HudStatus = "idle" | "listening" | "thinking" | "speaking";

export function HudCard({
  status = "idle",
  children,
}: {
  status?: HudStatus;
  children?: ReactNode;
}) {
  return (
    <div className="flex h-screen w-screen items-start justify-end p-0">
      <section
        className="flex w-full flex-col gap-3 rounded-2xl border border-white/10
                   bg-slate-900/40 p-4 text-slate-100 shadow-2xl backdrop-blur-xl"
      >
        <header className="flex items-center justify-between text-sm tracking-wide">
          <span className="font-semibold text-[#22d3c5]">echo</span>
          <span className="uppercase text-slate-400">{status}</span>
        </header>
        <div className="min-h-24 text-sm text-slate-300/80">
          {children ?? "Ready when you are."}
        </div>
      </section>
    </div>
  );
}
