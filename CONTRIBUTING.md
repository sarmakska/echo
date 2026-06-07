# Contributing to Echo

Echo is built phase by phase (see `ROADMAP.md`). Do not start a phase until the
prior phase is verifiably done.

## Local setup

- Node 25 (`nvm use`), pnpm 10, Rust stable, Tauri 2 prerequisites for your OS.
- `pnpm install`
- `pnpm dev` launches the shell with hot reload.
- `pnpm test` runs frontend tests; `pnpm test:rust` runs Rust tests.

## Principles

- Local-first. No telemetry, ever.
- No API keys — brains are subscription-backed CLIs.
- Plain, portable data (Markdown + JSONL).
- MIT licensed.

## Workflow

The canonical plan is `PLAN.md`; phase status lives in `ROADMAP.md`.
Tests come before implementation. Keep files small and single-purpose.
