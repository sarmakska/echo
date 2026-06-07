<div align="center">

<a href="https://www.sarmalinux.com/products/echo">
<img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=12,18,20&height=210&section=header&text=echo&fontSize=78&fontColor=ffffff&fontAlignY=50&desc=an%20open%20Jarvis%20on%20every%20screen&descSize=17&descAlignY=78&animation=fadeIn" width="100%" alt="echo, an open Jarvis on every screen"/>
</a>

<h1 align="center">echo</h1>

<p align="center">A personal AI assistant that runs on the subscription you already pay for. Voice in, voice out, vision when it helps, memory across years, and a translucent multi-monitor HUD. One Rust core, cross-platform. No API keys, no second bill, local-first.</p>

<p align="center"><em>echo is not affiliated with or endorsed by Anthropic, OpenAI or Google. Claude, ChatGPT and Gemini are trademarks of their respective owners, referenced here only to describe the command-line tools echo can dispatch to.</em></p>

<p align="center">
<a href="./LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/sarmakska/echo?color=38bdf8"></a>
<a href="https://www.rust-lang.org/"><img alt="Language: Rust" src="https://img.shields.io/github/languages/top/sarmakska/echo?color=22d3ee"></a>
<img alt="Tests" src="https://img.shields.io/badge/tests-90%20passing-34d399">
<a href="https://github.com/sarmakska/echo/commits/main"><img alt="Last commit" src="https://img.shields.io/github/last-commit/sarmakska/echo?color=22d3ee"></a>
<img alt="v1.0 target" src="https://img.shields.io/badge/v1.0-1%20July%202026-8b5cf6">
<a href="https://www.sarmalinux.com/products/echo"><img alt="Read the long plan" src="https://img.shields.io/badge/Read%20the%20long%20plan%20→-sarmalinux.com%2Fproducts%2Fecho-a78bfa"></a>
</p>

</div>

---

Every AI assistant app does at least one of these: locks you into one provider, sends your data through a server you do not own, charges a second subscription on top of the one you already pay, or breaks the moment a model gets renamed. echo refuses all four. It has no API of its own and never asks for an API key. It dispatches each prompt to whichever subscription-backed CLI you already pay for, `claude`, `codex` or `gemini`, chosen by a small router that scores capability, remaining quota and freshness. One subscription is the entire AI bill.

It is the assistant a privacy-aware engineer would actually trust to run in the background of their life.

## What it is

- **Bring your own subscription, never an API key.** echo shells out to the official `claude`, `codex` or `gemini` CLI you already pay for. No surprise overage, no second bill. One click in the setup wizard authorises whichever you have.
- **A brain-agnostic router.** Each prompt goes to the best-fit brain by capability, quota remaining and freshness. Pin one for a session or a kind of task; pins win.
- **A voice loop.** Wake word, speech to text, brain dispatch, streaming text to speech, with barge-in. Sub-second turn target on a modern laptop.
- **A glance-able HUD.** A translucent, always-on-top glass card, multi-monitor aware.
- **A skills bus.** Calendar, mail, files, web, music, notes, tasks, smart home and more, as MCP servers over stdio. Any MCP server in the wild plugs in unchanged. One-click OAuth for the cloud ones.
- **Memory across years.** Plain Markdown facts plus JSONL episodes plus a PreSession digest. If echo dies, your data does not.
- **macOS, Windows, Linux.** One Tauri 2 and Rust core. MIT licensed. No telemetry, ever.

The brain-authorisation philosophy lives in [ANY-BRAIN.md](./ANY-BRAIN.md). The full design lives in [PLAN.md](./PLAN.md), with diagrams in [ARCHITECTURE.md](./ARCHITECTURE.md).

## The seven non-negotiables

| Rule | Why |
| --- | --- |
| No API keys. Subscription-backed CLIs only. | Bounded cost, zero surprise overage. |
| Cross-platform: macOS, Windows, Linux from one codebase. | echo follows you from home Mac to work ThinkPad. |
| Multi-screen aware from day one. | Most knowledge workers have two or more displays. |
| Single-click OAuth, not config files. | Setup friction kills adoption. |
| Local-first. echo never phones home. | Brain CLIs talk to providers over channels you already use. Nothing else leaves the machine. |
| MIT licence. | Same standard as the rest of the sarmalinux.com projects. |
| No proprietary lock-in. Plain Markdown, JSONL, sqlite-vss; skills are plain Node or Rust. | If echo dies, your data does not. |

## What works today

echo is built phase by phase, and only ships a phase when its gate passes on real hardware. The Phase 0 foundation and the Phase 1 software are in and tested:

- Tauri 2 and pnpm monorepo with a translucent glass HUD card, placed top-right of the primary monitor.
- Brain router over the `claude`, `codex` and `gemini` CLIs, with capability and quota scoring and pins.
- File-based memory store: Markdown facts, daily JSONL episodes, recency recall, PreSession digests.
- MCP skills bus with three reference skills: weather, web search and local files.
- Voice loop with an energy-VAD wake fallback, a whisper.cpp speech-to-text adapter, and Piper plus macOS text to speech, with barge-in.
- An in-process agentic tool-call loop, and a first-launch setup wizard.

90 tests passing (80 Rust, 10 frontend). The live voice worker is wired behind a build feature; the day-one voice round trip turns green once the local binaries are installed (see the roadmap).

## Roadmap to v1.0 on 1 July 2026

Linear and gated. A phase is done only when its verification gate passes on real hardware. Full phase detail and status live in [ROADMAP.md](./ROADMAP.md).

| Phase | Scope | State |
| --- | --- | --- |
| 0 Foundation | Monorepo, CI, Tauri shell, glass HUD card | software in, verified on macOS |
| 1 MVP voice loop | Wake, mic, STT, ClaudeBrain, TTS, memory, three skills, wizard | software in and tested; hardware gate pending local install |
| 2 HUD polish | Full layout, multi-monitor roles, animations, accessibility, settings, tray | planned |
| 3 Multi-brain router | Codex and Gemini live, scoring, pins, quota in the HUD | brains and scoring built; live validation pending |
| 4 Calendar and mail | OAuth callback server, OS keychain, Google, Microsoft and Apple skills | planned |
| 5 The senses | Vision, music, notes, tasks, health, Home Assistant, Slack | planned |
| 6 Proactive engine | Cron and watch scheduler, quiet hours, focus mode, approval queue | planned |
| 7 Autonomous tasks | Multi-step workflows, approval queue in the HUD | planned |
| 8 Packaging and release | Signed DMG, signed MSI, Linux AppImage and deb and rpm, auto-update | planned |

The first tagged public release, **v1.0.0, is planned for 1 July 2026**, when the day-one target works end to end on macOS, Windows and Linux:

> Say "Echo, what is the weather in Hemel today." The wake word triggers, the HUD pulses, a transcript appears, the brain replies, and TTS speaks the forecast. Total round trip under three seconds on a midrange laptop.

## Status

Pre-release, building toward v1.0 on 1 July 2026. APIs and layout will move between now and then. No release tag is cut yet; the first will be v1.0.0.

## License

MIT. Built by [sarmalinux](https://www.sarmalinux.com) in Hemel Hempstead, UK.

Full plan, updates and the long write-up: [sarmalinux.com/products/echo](https://www.sarmalinux.com/products/echo).
