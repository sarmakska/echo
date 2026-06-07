# Brain Router

The single most important component. Get it right and Echo feels intelligent. Get it wrong and Echo feels like a remote control for whichever brain ran first.

## The premise

The user pays for one or more of:

- Claude Pro / Max (Anthropic, `claude` CLI)
- ChatGPT Plus / Pro / Team (OpenAI, `codex` CLI)
- Gemini Pro / Code Assist (Google, `gemini` CLI)

Echo dispatches each request to the brain best suited for the job, given:

1. **Capability fit**: does the brain do this kind of task well
2. **Quota left this window**: are we close to the rate limit
3. **Cost in latency**: cold subprocess vs warm cache
4. **User pin**: did the user say "use Claude for the next hour"

## Capability matrix

| Capability | Claude | Codex | Gemini |
|---|---|---|---|
| Long context (200k+) | yes | partial | yes (1M) |
| Streaming tokens | yes | yes | yes |
| Tool use / MCP | yes (native) | yes (MCP-shape) | yes (function calling) |
| Vision | yes | yes | yes |
| Image generation | no | yes (Pro) | yes (Imagen) |
| Code reasoning | strong | strong | medium |
| Web grounding (built-in) | no | partial | yes |
| Multi-turn memory across sessions | external (slipstream) | external | external |
| Hooks (PreCompact / SessionStart) | yes | no | no |
| Long sustained task | strong | medium | medium |

This drives the capability-match score.

## Scoring formula

For every brain `b` in the enabled set, the router computes:

```
score(b) = capability_match(b, task) * 100
         + quota_remaining_pct(b) * 30
         + freshness_bonus(b) * 10
         + user_pin_bonus(b)
```

Where:

- `capability_match` is a 0..1 score from the capability matrix
- `quota_remaining_pct` is 0..1, fetched from the brain's `--quota` style endpoint (each CLI exposes a remaining-budget hint)
- `freshness_bonus` is 1 if the brain subprocess is already warm, 0 otherwise
- `user_pin_bonus` is a large constant (10000) when the user has pinned the brain for this kind of task or session

The brain with the highest score wins. Ties go to the lower-cost brain (cheapest plan first).

## Pin semantics

| Pin | Lifetime | Example |
|---|---|---|
| Session pin | Until app restart or user un-pins | "Echo, only use Claude for the next hour" |
| Task pin | Forever, for a class of request | "Always use Gemini for web research" |
| Skill pin | When a specific skill is invoked | Vision tasks pinned to Codex when configured |

Pins are stored in `~/.echo/brains.yml`.

## Subprocess lifecycle

Each brain has a single warm subprocess held by a worker. The router queues prompts to the worker.

- **Warm state**: subprocess running, stdin open, last activity < idle timeout (default 90s).
- **Cold start**: spawn the binary, run a tiny health check, mark warm.
- **Idle reap**: after `idle_timeout` seconds without a request, kill the subprocess to free RAM.
- **Crash recovery**: subprocess exits unexpectedly. The worker logs, marks brain offline for 30 seconds, falls back to the next-best brain for any pending request.

## Quota awareness

Each CLI has a different way to report remaining budget:

| Brain | Way to read quota |
|---|---|
| `claude` | `claude doctor` returns `Pro: 234/300 messages this 5h window` |
| `codex` | `codex usage` returns `120 remaining of 300 daily` (placeholder; check current CLI) |
| `gemini` | `gemini status --json` returns daily request count |

The router polls each brain's quota at most once a minute, caches the result, and uses it in the scoring formula. If quota is exhausted on the winning brain, the router falls through to the next-best.

## Conversation continuity

Each brain maintains its own thread context internally. To avoid context fragmentation across brains, Echo keeps the canonical conversation history in the memory store, and rebuilds the context envelope for each request from there. The brain's own internal context is treated as ephemeral.

This means switching mid-conversation between Claude and Gemini is a clean handoff: the next request to Gemini will include the full Claude-side history, rebuilt from Echo's memory.

## Tool calls

When the brain emits a tool call, the router:

1. Pauses streaming.
2. Routes the tool call to the skill bus.
3. Waits for the skill response.
4. Feeds the response back to the brain.
5. Resumes streaming.

The brain CLIs all support MCP-shape tool calls. The router speaks MCP natively so this is plumbing, not translation.

## Failover

If the chosen brain returns an error or hangs longer than the per-task timeout (default 30s), the router:

1. Cancels the subprocess request (sends SIGTERM, then SIGKILL after 1s).
2. Picks the next-best brain by score.
3. Restarts the request with the same context envelope.
4. Marks the failed brain `degraded` for 60s so it does not get picked again immediately.

This is the same shape as the failover in `sarmakska/Sarmalink-ai`. Echo borrows the proven pattern.

## Configuration

```yaml
# ~/.echo/brains.yml
default_brain: claude
idle_timeout_seconds: 90
per_task_timeout_seconds: 30
failover_cooldown_seconds: 60

brains:
  claude:
    enabled: true
    binary: claude
    extra_args: []
  codex:
    enabled: false
    binary: codex
    extra_args: []
  gemini:
    enabled: false
    binary: gemini
    extra_args: []

pins:
  - kind: skill
    skill: vision-screen
    brain: codex
  - kind: task_pattern
    pattern: "(research|find|search the web|look up)"
    brain: gemini
```

## Observability

Every router decision is logged to `~/.echo/logs/router.log`:

```
2026-06-02T14:32:11Z task=voice utterance="what's on today"
  candidates=[claude:154, codex:120, gemini:96] winner=claude reason=pin
2026-06-02T14:32:12Z brain=claude subprocess=warm prompt_tokens=842
2026-06-02T14:32:13Z brain=claude first_token_ms=212 total_ms=1480
```

This is what we tune from in the early weeks.
