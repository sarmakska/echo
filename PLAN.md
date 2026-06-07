# Echo

> A personal AI assistant for grown-ups. Voice in, voice out, vision when it helps, memory across years, ten thousand tools at its disposal. Lives on your desk, sees your screens, never sends data through a third party you do not trust. Runs on a Claude Pro, a ChatGPT Plus, or a Gemini Pro subscription. No API keys. No bills you did not see coming.

This document is the full plan. Read it once before writing a single line of code. Update it when the architecture moves.

---

## 1. What Echo is

Echo is a cross-platform desktop assistant that combines four ideas that have always sat in separate apps:

1. **A voice loop**. You speak. It listens. It thinks. It speaks back. Sub-second turn time on a modern laptop.
2. **A multi-brain router**. You bring your own Pro subscription (Claude, ChatGPT, Gemini). Echo dispatches each prompt to whichever brain is best for the task, with whichever has budget left this hour.
3. **A glanceable HUD**. A translucent overlay on your screens. Live transcript, current task, next meeting, weather, system health, anything you wire in. Multi-monitor aware.
4. **A skills bus**. Calendar, mail, browser, files, code, music, smart home, fitness, finance. One-click OAuth for the cloud services. AppleScript / DBus / PowerShell bridges for the native ones.

The childhood dream is Iron Man. The grown-up version is "AI that respects my privacy, my wallet, and my time."

---

## 2. The non-negotiable constraints

These are the rules. If a design choice breaks any of them, the choice is wrong.

| Rule | Why |
|---|---|
| **No API keys.** Use subscription-backed CLIs only. | The user is paying $20 a month and that is the budget. No surprise overage bills. |
| **Cross-platform.** macOS, Windows, Linux. | The user might be on a Mac at home and a Windows ThinkPad at work. Echo follows them. |
| **Multi-screen aware.** | Most knowledge workers have 2+ displays. The HUD should make use of them, not pretend they are one. |
| **Single-click integrations.** OAuth flows, not config files. | Setup friction kills adoption faster than missing features. |
| **Local-first.** No data leaves your machine unless you authorise the service. | The brain CLIs talk to their providers (Anthropic, OpenAI, Google) over the same channels you would use anyway. Echo itself does not phone home. |
| **MIT license.** Wiki, whitepaper, architecture diagrams, demo videos. | Same standard as the other eighteen sarmalinux.com repos. |
| **No proprietary lock-in.** Memory store is plain Markdown + JSONL. Skills are plain Node or Rust crates. | If Echo dies, your data does not. |

---

## 3. The three-brain router

The headline trick. Three subscription-backed CLIs exist as of mid-2026:

| Provider | CLI binary | Subscription required | What it is best at |
|---|---|---|---|
| Anthropic | `claude` (Claude Code) | Pro $20/mo or Max $100/mo | Long context, code, sustained reasoning, tool use, hooks, MCP |
| OpenAI | `codex` (Codex CLI) | Plus $20/mo, Pro, Team, Enterprise | Mixed code/text, image gen on Pro, occasional unique reasoning |
| Google | `gemini` (Gemini CLI) | Free tier OR Gemini Code Assist Individual ($19/mo) OR Pro | Native web grounding, long-context summarisation, Workspace integration |

Echo expects the user to have one of the three at minimum. If they have more than one, Echo learns to dispatch by the **brain registry**.

### 3.1 Brain registry

```yaml
# ~/.echo/brains.yml
brains:
  claude:
    enabled: true
    binary: claude
    auth_check: claude doctor
    cost_tier: subscription_pro
    capabilities: [code, reason, tool_use, mcp, long_context, vision]
    quota_window: 5h_300_msg
  codex:
    enabled: false
    binary: codex
    auth_check: codex auth status
    cost_tier: subscription_plus
    capabilities: [code, reason, vision, image_gen, web_search]
    quota_window: 5h_120_msg
  gemini:
    enabled: false
    binary: gemini
    auth_check: gemini status
    cost_tier: subscription_pro
    capabilities: [reason, long_context, web_grounding, workspace, vision]
    quota_window: 24h_1000_req
```

### 3.2 Router decision policy

The router picks a brain per request using a simple scoring function:

```ts
score = capability_match * 100
      + quota_remaining_pct * 30
      + freshness_of_response_ms_inverse * 10
      + user_pin_bonus
```

The user can pin a brain for a session ("only use Claude for the next hour") or for a kind of task ("always use Gemini for web research"). Pins win.

### 3.3 The brain interface

Every brain conforms to the same Rust trait:

```rust
#[async_trait]
pub trait Brain {
    fn id(&self) -> &str;
    fn capabilities(&self) -> &[Capability];
    async fn quota(&self) -> Quota;
    async fn ask(&self, prompt: Prompt, ctx: Context) -> Result<Response, BrainError>;
    async fn stream(&self, prompt: Prompt, ctx: Context) -> Result<TokenStream, BrainError>;
}
```

Each concrete implementation is a thin subprocess wrapper around the matching CLI:

- `ClaudeBrain` shells out to `claude --print --append-system-prompt "$ECHO_SYS" --output-format stream-json`
- `CodexBrain` shells out to `codex exec --stream --json`
- `GeminiBrain` shells out to `gemini --prompt-file <stdin> --json`

Output is normalised to a common `Response` envelope. The shell is a clean subprocess each turn, no hidden persistent state in the CLI.

### 3.4 Why subprocess, not API

1. The user is paying for the subscription anyway. Going via the official CLI runs on the same quota the user is paying for. Zero surprise bills.
2. Each CLI handles its own auth, OAuth refresh, rate limits, and prompt caching. We do not reinvent any of that.
3. If the provider changes their API tomorrow, the CLI tracks it. We do not have to.

---

## 4. The voice loop

Three sub-loops running in parallel, talking via channels.

```
mic -> VAD -> wake-word -> STT -> brain router -> TTS -> speakers
                                        |
                                        +-> HUD updates
                                        +-> memory writes
                                        +-> skill invocations
```

### 4.1 Wake word

`porcupine` (Picovoice). Personal use free tier covers the "echo" or "jarvis" wake word with low CPU. Runs on macOS, Windows, Linux. Latency ~30ms.

Fallback for fully offline / Linux without GPU: simple VAD that triggers on any speech, with a 600ms silence-end detect.

### 4.2 Speech to text

`whisper.cpp` running locally. Models stored in `~/.echo/models/`. Default `small.en` for English on a midrange laptop, `base.en` for the lowest spec, `medium.en` for accuracy on a Pro spec.

Streaming with `--vad-thold 0.3`. First-token-out latency under 400ms on Apple Silicon.

### 4.3 Brain dispatch

Once a full utterance is in, the router picks a brain (see section 3) and dispatches with:

```ts
const ctx = {
  recentTurns: memory.recent(10),
  relevantFacts: memory.recall(utterance, budget=2000),
  activeSkills: skills.activeFor(utterance),
  multiScreenState: hud.observableState(),
  userProfile: profile.read(),
}
```

### 4.4 Streaming text to speech

As tokens arrive from the brain, split on sentence boundaries, stream each sentence to a TTS endpoint. Same pattern Echo's older sibling (sarmalinux.com chatbot) already ships.

TTS backends, in order of preference:

1. **Local Piper** TTS (free, MIT, decent quality). The default everywhere.
2. **macOS** native `AVSpeechSynthesizer` (zero install).
3. **Windows** SAPI5 (zero install).
4. **Linux** `espeak-ng` (apt install).
5. **Optional Cloudflare Workers AI** MeloTTS for higher quality, if the user has their own free Cloudflare account configured (one-click setup).

User picks a voice profile. Echo defaults to a neutral British female. Customisable in settings.

### 4.5 Barge-in

While Echo is speaking, the mic still listens. If the user starts talking, TTS stops mid-sentence and the new utterance becomes the next prompt. Crucial for natural conversation, hard to get right. Reuses the proven turn-state machine from sarmakska/voice-agent-starter.

---

## 5. The HUD

The visible part. A translucent always-on-top overlay rendered with Tauri 2. Multi-monitor aware from day one.

### 5.1 Layout philosophy

Glance, do not stop. Every panel must communicate in under one second of attention. Nothing that requires reading a paragraph.

### 5.2 Default layout (single monitor)

```
+-----------------------------------------------+
| top-right corner, 380x540, glass card         |
| +-------------------------------------------+ |
| | [echo]   23 Aug, 14:32   HEMEL  *recording* |
| +-------------------------------------------+ |
| | live transcript scrolls here              | |
| |                                           | |
| +-------------------------------------------+ |
| | NEXT  Standup, 5 min                      | |
| | TODAY 12C clear, rain at 18:00            | |
| | INBOX 3 unread                            | |
| | CPU  21%   MEM 8.2/16  NET 41kb/s         | |
| | BRAIN claude (Pro)  234/300 left          | |
| +-------------------------------------------+ |
+-----------------------------------------------+
```

### 5.3 Multi-monitor layout

Echo detects connected displays at startup and on hot-plug. The user assigns roles:

| Role | Default panel | Example use |
|---|---|---|
| `primary` | Glass HUD card | Main monitor where you do work |
| `dashboard` | Full system metrics, agent queue, today's calendar | A second monitor as a wall display |
| `transcript` | Big live transcript and TTS playback | A third monitor for the conversation feed |
| `ambient` | Slow rotating brief (weather, news, finance, fitness) | A side iPad-as-display showing the room status |

Layout config:

```yaml
# ~/.echo/displays.yml
displays:
  "Built-in Retina":
    role: primary
    position: top_right
    width_px: 380
  "DELL P2723D":
    role: dashboard
    fullscreen: false
    panels: [metrics, calendar, queue]
  "LG UltraFine":
    role: ambient
    fullscreen: true
    rotation_secs: 12
```

### 5.4 Pulse animations

Listening = soft pulsing blue ring around the wake-word indicator. Thinking = clockwise spinner sweep. Speaking = waveform-of-the-current-sentence at the bottom of the card. Idle = gentle 4-second breathing dim/brighten.

### 5.5 Accessibility

Keyboard-only mode for users who do not want to speak. Press `Ctrl+Space` to bring up an inline command bar in the HUD; type the prompt; Echo replies in text and TTS.

Subtitle mode for users who want to read while listening, or for noisy environments. Captions stream into the HUD card.

---

## 6. The skills bus

Skills are how Echo acts on the world. Each skill is a small package with a manifest, a Rust adapter or a Node/Python MCP server, and a set of tool definitions.

### 6.1 Skill manifest

```yaml
# packages/skills/calendar-google/skill.yml
name: calendar-google
version: 0.1.0
display: Google Calendar
icon: /icons/google-calendar.svg
auth:
  kind: oauth2
  provider: google
  scopes: [calendar.readonly, calendar.events]
tools:
  - name: list_events
    args: { date: string }
  - name: create_event
    args: { title: string, start: datetime, end: datetime, attendees?: string[] }
  - name: cancel_event
    args: { event_id: string }
proactive:
  - cron: "0 7 * * *"
    handler: morning_briefing
  - watch: calendar.events.upcoming_15m
    handler: meeting_reminder
```

### 6.2 The standard skill catalogue

Phase 1 ships with these. Each is one-click connect.

| Skill | Auth | What it does |
|---|---|---|
| `calendar-google` | OAuth Google | Read + create + cancel events, attendees |
| `calendar-microsoft` | OAuth Microsoft | Same, for Outlook 365 |
| `calendar-apple` | macOS EventKit (no auth) | Local Apple Calendar read + write |
| `mail-google` | OAuth Google | Read recent, draft, send, summarise unread |
| `mail-microsoft` | OAuth Microsoft | Same for Outlook |
| `mail-apple` | macOS AppleScript | Local Mail.app via osascript |
| `files-local` | None | Read, write, search local files |
| `web-search` | None | Brave/Tavily/SearXNG public APIs, free tier |
| `vision-screen` | OS permission | Screenshot any monitor, send to vision-capable brain |
| `music-spotify` | OAuth Spotify | Play, pause, skip, queue, search |
| `music-apple` | macOS AppleScript | Apple Music control |
| `home-assistant` | Long-lived token | Toggle lights, scenes, temperature |
| `health-apple` | macOS Shortcuts | Read step count, heart rate, sleep |
| `health-google-fit` | OAuth Google | Same for Android users |
| `notes-notion` | OAuth Notion | Create page, append to page, search |
| `notes-obsidian` | None (local vault) | Read/write Markdown vault |
| `tasks-linear` | OAuth Linear | Create issue, list mine, change status |
| `tasks-github` | OAuth GitHub | Create issue, comment, draft PR |
| `chat-slack` | OAuth Slack | Read DMs, post in channel, status update |
| `bank-plaid` | OAuth Plaid (optional) | Balance, recent transactions, budget snapshot |
| `news` | None | RSS / Hacker News / BBC / chosen feeds |
| `weather` | None | Open-Meteo, free, no key |

Each skill is independently installable, updatable, removable. The skill catalogue UI in the HUD shows installed skills, connection status, last-used timestamp, and a one-click connect/disconnect button.

### 6.3 Single-click OAuth

For every OAuth-based skill, Echo runs a tiny local OAuth callback server on a random port between 50000 and 60000, opens the provider's auth URL in the system browser, captures the redirect, stores the refresh token in the OS keychain (Keychain on Mac, Credential Vault on Windows, Secret Service on Linux). Never written to disk in plain text.

Token refresh is automatic. If a refresh fails, the skill is marked `needs_reconnect` and shows a yellow dot in the HUD until the user clicks reconnect.

### 6.4 Skill API contract

Skills run as MCP servers behind the scenes. Echo's skill bus talks to them over stdio JSON-RPC, the same protocol Claude Code already uses. This means:

- Any existing MCP server in the wild plugs into Echo with zero adaptation.
- Echo skills can be installed into Claude Code, Codex, or any other MCP-aware host without modification.
- The contract is documented and the skill SDK is plain TypeScript or Rust.

---

## 7. Memory

Long-lived knowledge. Echo remembers what you told it last month so you do not have to repeat yourself.

### 7.1 Store layout

```
~/.echo/memory/
  facts/                   # one Markdown file per durable fact
    sarma_lives_hemel.md
    sarma_uses_claude_pro.md
    project_echo.md
    ...
  episodes/                # one JSONL per day of conversation
    2026/06/02.jsonl
  digests/                 # PreCompact-style structured digests per session
    session_2026-06-02-1432.md
  index.md                 # auto-regenerated index of all facts
  embeddings.sqlite        # local vector index (FAISS or sqlite-vss)
```

Same architecture as the user's `slipstream` repo. Echo borrows the proven pattern.

### 7.2 Recall

Two paths, used in tandem:

1. **Signal-ranked recall**. For each turn, rank all facts by relevance to the current utterance (vector similarity + recency). Take up to a token budget (default 2000 tokens) and inject as system context.
2. **PreSession digest**. Each session starts by reloading the most recent digest from the previous session, so context survives quitting and restarting.

### 7.3 Forget

The user can say "Echo, forget what I said about X" or open the HUD and delete a fact directly. Deletion removes the fact and re-indexes. Undo for 30 days.

### 7.4 Profile

A small structured `profile.yml` Echo always loads:

```yaml
name: Sarma
display_name: Sarma
location: Hemel Hempstead, Hertfordshire, UK
timezone: Europe/London
preferred_voice: en_GB_female
preferred_brain: claude
working_hours: 09:00-18:00
quiet_hours: 22:00-07:00
```

---

## 8. Proactive engine

Iron Man's Jarvis does not wait for prompts. He anticipates. Echo does this with a tiny cron-style scheduler.

### 8.1 The watch list

```yaml
# ~/.echo/proactive.yml
watches:
  - id: morning_brief
    when: cron("0 7 * * 1-5")
    skill: calendar+mail+weather+news+finance
    deliver: voice_brief
  - id: meeting_warn
    when: skill_event("calendar.event.in_10m")
    deliver: chime + voice("Standup in ten minutes")
  - id: long_idle
    when: idle_for(75m) && working_hours
    deliver: voice("You have been heads down for over an hour, you might want a break.")
  - id: deploy_failure
    when: github_actions.failure(repo="sarmakska/*")
    deliver: chime + voice("Build failed on $repo: $job")
  - id: charity_dm_burst
    when: facebook.dms.new_in_last_15m > 3
    deliver: voice("Three new charity-offer DMs in the last quarter hour, do you want me to draft replies?")
```

### 8.2 Quiet hours and focus mode

Outside working hours Echo is silent unless the watch is marked `priority: emergency`. During screen-sharing or video calls Echo enters focus mode automatically (detected by OS APIs), suppresses voice output, and queues anything in the HUD only.

---

## 9. Cross-platform architecture

### 9.1 Tech stack

| Layer | Choice | Why |
|---|---|---|
| Shell | Tauri 2 (Rust + Web frontend) | Mac, Windows, Linux from one codebase. Sandboxed. Tiny binary. |
| Frontend | React 19 + Tailwind v4 | Same stack as sarmalinux.com, fast iteration. |
| Audio in | `cpal` (Rust) | Cross-platform mic capture without WebRTC dependencies. |
| Audio out | OS native + Piper fallback | Best quality for each OS. |
| Wake word | Porcupine (cross-platform SDK) | Industry standard, low CPU. |
| STT | whisper.cpp via Rust binding | Self-contained, runs offline. |
| Brain CLIs | Subprocess invocation | Claude/Codex/Gemini binaries are the user's. |
| Skill bus | MCP over stdio | Industry-standard, future-proof. |
| Memory | Markdown + JSONL + sqlite-vss | Plain, portable, auditable. |
| OAuth | Local callback server + OS keychain | Standard, secure, no third party. |
| Crons | Tauri scheduler + OS-native task scheduler | Reliable across OSes. |

### 9.2 Why Tauri over Electron

- 10x smaller binary (~10MB vs 100MB+)
- Rust gives us proper cross-platform native APIs (multi-display detection, OS keychain, native menus, hot-plug events)
- Sandboxed by default, security model fits the local-first promise
- Mature multi-window support for the HUD

### 9.3 Packaging

| OS | Format | Build |
|---|---|---|
| macOS | `.dmg`, signed + notarised | `tauri build` on macOS runner |
| Windows | `.msi` and `.exe`, signed | `tauri build` on Windows runner |
| Linux | `.deb`, `.rpm`, `.AppImage`, Flatpak (later) | `tauri build` on Linux runner |

CI in GitHub Actions builds and publishes all three on tag push.

### 9.4 Auto-update

Tauri's built-in updater. Pulls from `releases/echo` on GitHub. Signed manifests so a compromised CDN cannot push malware.

---

## 10. Security and privacy

Echo's promise is local-first. The threat model has to back that up.

### 10.1 Trust boundaries

| Boundary | What crosses it |
|---|---|
| User → Echo | Voice, text, screen content, keychain unlock prompt |
| Echo → brain CLI | Single prompt + context window (subprocess stdin) |
| Echo → skill | Tool call args (stdio JSON-RPC to the MCP server) |
| Skill → upstream service | The OAuth-authorised calls the skill is meant to make |
| Echo → disk | Memory store, encrypted at rest if user enables it |
| Echo → network | Only via skills the user enabled |

### 10.2 Specific controls

- Mic toggle in the HUD, hardware-style "mute" indicator. Off means off, never bypassable by skills.
- Screen capture requires explicit per-session permission. macOS Screen Recording entitlement, Windows similar. Linux uses XDG portal.
- All OAuth tokens in OS keychain, never on disk in plain text.
- Optional disk encryption for `~/.echo/memory/` using OS-level secure storage as the key wrap.
- "Forget" button in the HUD wipes all memory and logs for the current session, and a "Forget everything" button wipes the whole store.
- Network calls logged with skill and target host. User can view the audit log in the HUD.

### 10.3 Threat model

We protect against:

- A compromised skill exfiltrating data (sandboxed subprocess, network-policy declared in skill manifest, blocked by default)
- A malicious update (signed manifests, rollback on signature failure)
- A local attacker stealing tokens (OS keychain, not on disk)
- A network attacker (HTTPS everywhere, certificate pinning for brain CLIs would be too brittle so we rely on the OS trust store)

We do NOT protect against:

- The user's brain provider misusing data they already get via the subscription
- The user explicitly granting a skill broad scopes and that skill misbehaving (user audit responsibility)
- Physical access to an unlocked laptop

---

## 11. Repo layout

```
echo/
├── apps/
│   ├── shell/                    # Tauri 2 app, HUD windows, system tray, hot keys
│   ├── voice/                    # Wake-word + STT loop, cross-platform
│   └── brain/                    # Router + brain implementations (Claude/Codex/Gemini)
├── packages/
│   ├── skill-sdk/                # TypeScript SDK for writing skills
│   ├── skill-sdk-rs/             # Rust SDK for writing skills
│   ├── memory/                   # File-based memory store + recall
│   ├── ui/                       # Shared React components for the HUD
│   ├── ipc/                      # JSON-RPC types shared across processes
│   └── proactive/                # Cron + watch scheduler
├── skills/                       # First-party skills (one folder each)
│   ├── calendar-google/
│   ├── calendar-microsoft/
│   ├── calendar-apple/
│   ├── mail-google/
│   ├── mail-microsoft/
│   ├── mail-apple/
│   ├── files-local/
│   ├── web-search/
│   ├── vision-screen/
│   ├── music-spotify/
│   ├── music-apple/
│   ├── home-assistant/
│   ├── health-apple/
│   ├── notes-notion/
│   ├── notes-obsidian/
│   ├── tasks-linear/
│   ├── tasks-github/
│   ├── chat-slack/
│   ├── news/
│   └── weather/
├── docs/
│   ├── PLAN.md                   # this file, the canonical plan
│   ├── ARCHITECTURE.md           # diagrams, request lifecycle, sequence flows
│   ├── BRAIN-ROUTER.md           # the dispatch policy in detail
│   ├── SKILLS.md                 # how to write a skill
│   ├── HUD.md                    # design language and layout rules
│   ├── MEMORY.md                 # store layout, recall algorithm
│   ├── SECURITY.md               # threat model, controls
│   └── ROADMAP.md                # phase list with done/in-progress/next
├── installers/                   # OS-specific install scripts and helper apps
│   ├── macos/
│   ├── windows/
│   └── linux/
├── .github/workflows/            # CI: build all three platforms on tag
├── LICENSE                       # MIT
├── README.md
└── package.json                  # pnpm workspace root
```

---

## 12. Phases

Be disciplined. Ship Phase 1 end-to-end before starting Phase 2.

### Phase 1, MVP, 2 weekends
- [ ] Repo scaffold, Tauri 2 shell, one HUD window
- [ ] Voice loop (wake word + STT + brain dispatch + TTS) with `claude` brain only
- [ ] Memory store with PreSession digest
- [ ] 3 skills: `weather`, `web-search`, `files-local`
- [ ] Single-monitor HUD layout, glass card top-right

Definition of done: walk into the office, say "Echo, what is on today" and hear a coherent reply.

### Phase 2, HUD polish, 2 weekends
- [ ] Translucent overlay with the full layout from section 5
- [ ] Multi-monitor detection and role assignment
- [ ] Pulse animations, accessibility mode, subtitle mode
- [ ] Settings panel in the HUD
- [ ] System tray with quick actions

### Phase 3, brain router, 1 weekend
- [ ] Add `codex` brain
- [ ] Add `gemini` brain
- [ ] Implement scoring policy
- [ ] Per-session and per-task pins
- [ ] Quota display in HUD

### Phase 4, calendar + mail, 2 weekends
- [ ] `calendar-google` skill with full OAuth
- [ ] `calendar-microsoft` skill
- [ ] `calendar-apple` skill via EventKit
- [ ] Same three for mail
- [ ] Morning brief proactive watch

### Phase 5, the senses, ongoing
- [ ] Vision (`vision-screen`)
- [ ] Music control
- [ ] Notes (Notion + Obsidian)
- [ ] Tasks (Linear + GitHub)
- [ ] Health (Apple Health + Google Fit)
- [ ] Home Assistant
- [ ] Slack

### Phase 6, proactive engine, ongoing
- [ ] Full cron + watch scheduler
- [ ] Quiet hours and focus mode
- [ ] User-defined watches in the settings UI
- [ ] Reusable watch templates ("warn me X minutes before any meeting", "summarise unread mail at Y o'clock")

### Phase 7, autonomous tasks, longer
- [ ] Integrate `agent-orchestrator` for multi-step workflows
- [ ] "Plan Mum's seventieth" template
- [ ] "Triage my unread mail and draft replies" template
- [ ] Approval queue in the HUD ("Echo is ready to send 4 drafts. Review.")

---

## 13. Open questions

These need answering before Phase 4 ships. Capture them, do not skip them.

1. **Multi-machine sync.** If the user has Echo on a Mac and a Windows machine, do their memories sync? Default proposal: no, each install is independent. Optional: encrypted CloudKit / OneDrive sync for power users. Decide before Phase 4.
2. **Mobile companion?** A small iOS / Android app that listens when the user is not at their desk and queues work for Echo to handle next time it sees them. Probably Phase 8+.
3. **Plugin marketplace.** Third-party skills, signed, reviewable. Probably after Phase 6.
4. **Voice cloning.** Should Echo learn to speak in the user's preferred voice from a sample? Tempting, but raises consent and deepfake questions. Default: no, ship preset voices only.
5. **Multi-user mode.** Does Echo recognise different speakers in the same room? Phase 7+ if at all.

---

## 14. The Sarma-specific touches

Because you are building this for yourself first.

- Defaults to British English everywhere. UK timezone. Hemel weather.
- Honours the PAYE-only stance: Echo will never volunteer to draft contract proposals or pricing emails, even if asked nicely. (Configurable, off by default.)
- Charity offer awareness: a watch for the Hemel charity DM inbox surfaces new applications, drafts replies, queues them for your review.
- Slipstream integration: Echo's memory store can be a slipstream backend. So the agent in Claude Code and Echo's voice loop share the same long-term memory.
- Tied to your VPS at `voice.sarmalinux.com` for premium TTS when the user opts in.
- MIT licence, no telemetry, never any "phone home" pings.

---

## 15. Naming and brand

- **Name**: Echo. Short, calm, not aggressive. Earns its keep over time.
- **Wake word**: configurable, default "Echo". User can change to "Jarvis", "Computer", anything supported by Porcupine plus a custom-trained one in a later phase.
- **Voice character**: by default neutral British female, calm, slight wit, never sycophantic. Configurable.
- **Visual identity**: glass, near-monochrome (slate ink + an accent of bioluminescent teal), Iron Man references but understated. Not blue glow everywhere.
- **Tone of messaging**: confident, technical, no fluff. Same voice as the rest of the sarmalinux.com brand.

---

## 16. Anti-features (explicitly NOT shipping)

So we do not creep.

- No cloud account that we run. Users bring their own provider subscriptions.
- No analytics, no telemetry, no "we collect anonymous usage data to improve". The repo is MIT and the user owns the device.
- No in-app purchases, no Pro tier. Echo itself is free and open.
- No social features. Echo is not a social network.
- No video generation, deepfakes, voice cloning, romantic-partner roleplay, or therapy mode. Out of scope.

---

## 17. The day-one verification target

When Phase 1 lands and you boot Echo for the first time, this exact sequence must work end to end on macOS, Windows, and Linux:

1. Run the installer.
2. First launch opens a setup wizard: pick a brain (Claude / Codex / Gemini), Echo runs the matching `--auth` check, confirms green.
3. Connect three skills: weather, web search, files. Each is one click.
4. Pick a wake word. Default "Echo".
5. Pick a voice. Default British female.
6. Wizard exits. HUD card appears top-right of the primary monitor.
7. Say "Echo, what is the weather in Hemel today."
8. Wake word triggers, HUD pulses, transcript appears, brain replies, TTS speaks the forecast.
9. Total round trip under three seconds on a midrange laptop.

If that works, Phase 1 is done. Cut a 0.1.0 release, write the README, push to GitHub, post to Hacker News, watch the inbox.

---

## 18. Why this is worth doing

There is no shortage of "AI assistant" apps. Every one of them either:

- Locks you into one provider, or
- Sends your data to a server you do not control, or
- Charges you a second subscription on top of the one you already pay, or
- Is a thin wrapper over a single API that breaks the moment the provider changes a model name.

Echo is the assistant you would actually trust to run in the background of your life. Local-first by design. Subscription-backed brains so the cost is bounded and predictable. Open source so anyone can audit what it does with their data. Cross-platform so it follows you. Multi-screen so it scales with how you actually work.

It is the version of Jarvis you can build today, on hardware you already own, with subscriptions you already pay for.

That is the project.
