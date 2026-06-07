# Echo Roadmap

Linear and gated. Do not start a phase until the prior phase is verifiably done on
real hardware. This is roughly two years of intended work, rolled out in small
increments rather than big-bang. New phases land continuously; the public history
shows the progression.

Target for the first tagged public release, **v1.0.0: 1 July 2026**, when the
day-one voice round trip works end to end on macOS, Windows and Linux. No release
tag is cut before then.

Status legend: [x] software in and tested, [ ] planned or pending its hardware gate.

## Phase 0, repo scaffold (this weekend)
- [x] `pnpm` monorepo, Tauri 2 shell skeleton
- [x] CI on GitHub Actions: build matrix Mac/Windows/Linux on push
- [x] License, README, CONTRIBUTING, SECURITY, CHANGELOG
- [x] First HUD window: blank glass card on the primary monitor
- [ ] Hot-reload dev loop confirmed on all three OSes (verified macOS; Windows/Linux pending)
- Verification gate: `pnpm dev` opens the HUD card top-right on macOS, Windows, Linux.

## Phase 1, MVP voice loop (weekends 2-3)
- [x] Wake-word listener: energy-VAD fallback (default); Porcupine adapter behind `wake` feature (needs key)
- [x] STT: `WhisperCliStt` driving whisper.cpp small.en (needs binary + model at runtime)
- [x] Brain worker, Claude only (`claude --print` subprocess)
- [x] Memory store (Markdown facts + JSONL episodes + PreSession digest)
- [x] TTS: `PiperTts` (default) + macOS `say` adapter wired into the shell `speak` command
- [x] 3 skills: weather, web search, files
- [x] Setup wizard that runs on first launch
- [x] In-process agentic tool-call loop (brain -> SkillRegistry -> result fed back)
- [x] `VoiceLoop` driver with barge-in (tested with fakes)
- [x] Live voice worker assembled in the shell (cpal mic -> wake -> STT -> TurnEngine -> TTS), behind `--features voice`; emits `echo://transcript` and `echo://reply` to the HUD
- [x] Skills exposed as an MCP server (PLAN 6.4) with `skill.yml` manifests
- [ ] Codex/Gemini stream parsers validated against the real CLIs (needs the CLIs installed)
- Definition of done: walk in, say "Echo, what is on today" and hear a coherent reply within 3 seconds on a midrange laptop.
- Software gate (passes now, macOS): `cargo test --workspace` 80 passed; `pnpm -r test` 10 passed; `cargo check -p echo-voice --features mic` and `cargo check -p echo-shell --features voice` both Finished.
- Hardware gate (pending Sarma's local install of claude + whisper.cpp + small.en + piper): the day-one round trip above, under 3 seconds, on macOS, Windows, Linux.

## Phase 2, HUD polish (weekends 4-5)
- [ ] Translucent always-on-top glass card with the standard layout
- [ ] Multi-monitor detection and role assignment (primary, dashboard, transcript, ambient)
- [ ] Pulse animations (listening, thinking, speaking)
- [ ] Accessibility mode (keyboard-only)
- [ ] Subtitle mode (live captions)
- [ ] System tray with quick actions (mute mic, mute TTS, open settings)
- [ ] Settings panel inside the HUD

## Phase 3, brain router (weekend 6)
- [ ] `codex` brain implementation
- [ ] `gemini` brain implementation
- [ ] Capability + quota scoring policy
- [ ] Per-session and per-task pinning
- [ ] Brain status row in the HUD

## Phase 4, calendar + mail (weekends 7-8)
- [ ] OAuth local callback server, keychain integration
- [ ] `calendar-google`, `calendar-microsoft`, `calendar-apple`
- [ ] `mail-google`, `mail-microsoft`, `mail-apple`
- [ ] Morning briefing proactive watch
- [ ] Meeting reminder proactive watch

## Phase 5, senses (ongoing)
- [ ] Vision (screen capture, vision-capable brain)
- [ ] Music (Spotify, Apple Music)
- [ ] Notes (Notion, Obsidian local vault)
- [ ] Tasks (Linear, GitHub)
- [ ] Health (Apple Health via Shortcuts, Google Fit)
- [ ] Home (Home Assistant)
- [ ] Slack
- [ ] News + finance + RSS

## Phase 6, proactive engine (ongoing)
- [ ] Cron-style scheduler with quiet hours and focus mode
- [ ] Settings UI for user-defined watches
- [ ] Reusable watch templates
- [ ] Approval queue in the HUD for drafts and outbound actions

## Phase 7, autonomous workflows (longer)
- [ ] Integrate the `agent-orchestrator` pattern from sarmakska/agent-orchestrator
- [ ] Multi-step task templates ("plan an event", "triage mail", "research a role")
- [ ] Long-running workflow inspector
- [ ] Persistent journal across restarts

## Phase 8, packaging and release (when Phase 1 is solid)
- [ ] Signed and notarised macOS DMG
- [ ] Signed Windows MSI
- [ ] Linux AppImage + deb + rpm
- [ ] Tauri auto-updater with signed manifests
- [ ] Public docs site (subdomain on sarmalinux.com)
- [ ] First public release 0.1.0

## Anti-roadmap (we will not do these)
- Cloud sync of memory (Phase 8+ optional, opt-in only, encrypted)
- Mobile companion (likely Phase 10+)
- Plugin marketplace (after Phase 6)
- Voice cloning
- Multi-user mode
- In-app purchases / Pro tier
- Telemetry of any kind
