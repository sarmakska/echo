# Changelog

All notable changes to Echo are documented here.
Format loosely follows Keep a Changelog. Versions follow SemVer.

## [Unreleased]

### Added
- Phase 0 scaffold: pnpm monorepo, Tauri 2 shell, translucent glass HUD window
  positioned top-right of the primary monitor.
- Phase 1 brains: `echo-brain` with the `Brain` trait, `ClaudeBrain`/`CodexBrain`/
  `GeminiBrain` subprocess wrappers, stream-json parser, and the capability/quota
  router (scoring plus pins).
- Phase 1 memory: `echo-memory` with Markdown facts, daily JSONL episodes,
  recency recall, and PreSession digests.
- Phase 1 orchestrator: `echo-core` `TurnEngine` (recall, ask, journal, reply)
  and the in-process `AgentLoop` (brain tool call to skill dispatch and back).
- Phase 1 skills: `echo-skills` MCP-style contract plus `files-local`, `weather`
  (Open-Meteo) and `web-search` (SearXNG) skills.
- Phase 1 voice: `echo-voice` trait contracts, fakes, sentence splitter, the
  `VoiceLoop` driver with barge-in, an energy-VAD wake fallback, a whisper.cpp
  CLI STT adapter, a Piper TTS adapter, a macOS `say` adapter, and feature-gated
  real cpal mic (`mic`) and Porcupine wake (`wake`) adapters.
- Phase 1 skills bus over MCP: `McpServer` exposes the `SkillRegistry` via
  stdio JSON-RPC (`initialize`, `tools/list`, `tools/call`); `skill.yml`
  manifests for the three Phase 1 skills.
- Shell: memory-aware `echo_turn`, `speak`, and `start_listening` commands; the
  live voice worker (cpal mic, VAD wake, whisper.cpp STT, TurnEngine, macOS TTS)
  behind `--features voice`, emitting `echo://transcript` and `echo://reply`; an
  interactive command bar; and the first-launch setup wizard.

### Local verification (run on macOS; Phase 0 plus Phase 1 software gate)
- `cargo test --workspace` => 80 passed, 0 failed.
- `cargo check -p echo-voice --features mic` => Finished (real cpal capture compiles).
- `cargo check -p echo-shell --features voice` => Finished (live voice worker compiles).
- `pnpm -r test` => 10 passed, 0 failed (frontend).
- `pnpm dev` => translucent glass HUD opens top-right; setup wizard on first launch;
  type a prompt in the command bar and, on macOS, hear the reply spoken.

### Pending before the Phase 1 hardware gate can pass
- Install `claude` so the command bar and `echo_turn` return real replies.
- Install the whisper.cpp binary plus the `ggml-small.en.bin` model in
  `~/.echo/models/`; install `piper` plus a British voice model for Piper output.
- For Porcupine wake: a Picovoice access key, then build with `--features wake`.
- Assemble the live voice worker thread (cpal mic -> wake -> STT -> TurnEngine ->
  TTS) inside the shell; validate the Codex and Gemini stream parsers against the
  real CLIs.
