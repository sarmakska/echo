# Any Brain Echo Can Drive

**No APIs. No keys. Only subscriptions you already pay for.**

This is the load-bearing principle. Echo never sends a request to an AI provider using an API key the user has to top up. Every brain Echo talks to is reached through the **same channel the user would use as a normal human paying customer**: an official CLI, a desktop client, or a sanctioned local agent.

This document codifies what counts, what does not, and how to add a new brain when one ships.

---

## Why this matters

A normal consumer pays $20 a month for a Pro subscription. They expect that to be all the AI costs they ever see. APIs charge per token on top, by a separate provider account, with bills that can spike to hundreds in a bad day. Echo refuses to put that risk on the user.

The pricing promise: **the only cost of running Echo is the AI subscription the user already chose to pay for, plus their electricity bill.**

---

## What counts as a brain

A brain is anything that conforms to the Echo Brain contract (see section 3 below) **and** is reached through a channel that the AI provider has sanctioned for consumer subscription use.

Sanctioned channels include:

1. **Official CLI** the provider ships and supports (e.g. `claude`, `codex`, `gemini`).
2. **Official desktop app** with a documented local IPC the provider published (e.g. some hosts expose a local socket for plugins).
3. **Local model runner** the user installed themselves (e.g. `ollama`, `lmstudio`) running a model the user has the right to use. This counts because there is no provider call at all.
4. **Provider-sanctioned MCP server** the provider ships that talks to their subscription back-end (if and when these exist).

Non-sanctioned channels do not count and are not supported:

1. Web scraping of `chat.openai.com`, `claude.ai`, `gemini.google.com`, or any other web product. Violates Terms of Service. Brittle. Out.
2. Reverse-engineered private APIs the provider has not blessed.
3. Headless browser automation impersonating a logged-in session.
4. Any third-party "free" gateway that resells provider access.

If a future provider does ship a sanctioned CLI or local agent, Echo grows a brain adapter for it.

---

## The current brain list

| Brain | Channel | Subscription | Status |
|---|---|---|---|
| `claude` | Claude Code CLI | Claude Pro / Max | Day-one |
| `codex` | Codex CLI | ChatGPT Plus / Pro / Team | Day-one |
| `gemini` | Gemini CLI | Google account / Gemini Code Assist Individual / Pro | Day-one |
| `ollama` | Local Ollama daemon | None (own hardware) | Day-one |
| `lmstudio` | Local LM Studio daemon | None (own hardware) | Phase 1 |

The list grows when new sanctioned consumer surfaces ship. Each addition is one adapter file, one entry in the registry, one settings card. No core changes.

---

## The brain contract

Every brain Echo supports implements this Rust trait. Implementations live in `apps/brain/src/brains/<brain-id>.rs`.

```rust
#[async_trait]
pub trait Brain: Send + Sync {
    /// Stable identifier, lowercase, kebab-case ("claude", "codex", "gemini", "ollama").
    fn id(&self) -> &'static str;

    /// Human-readable name shown in the HUD ("Claude Pro", "Codex (ChatGPT Plus)").
    fn display_name(&self) -> String;

    /// What this brain is good at, used by the router.
    fn capabilities(&self) -> &[Capability];

    /// Health and authentication check.
    /// Returns Ok(()) if the user is signed in and the channel is reachable.
    async fn check(&self) -> Result<BrainHealth, BrainError>;

    /// Best-effort quota estimate, used by the router.
    /// Implementations that cannot estimate return Quota::Unknown.
    async fn quota(&self) -> Quota;

    /// Send a prompt, get a streaming response.
    async fn stream(
        &self,
        prompt: Prompt,
        ctx: Context,
        cancel: CancellationToken,
    ) -> Result<TokenStream, BrainError>;
}
```

Capabilities are an open enum:

```rust
pub enum Capability {
    LongContext { window_tokens: u32 },
    ToolUse,
    Vision,
    ImageGen,
    WebGrounding,
    Code,
    Reasoning,
    StreamingTokens,
    Hooks,
    LocalOnly,         // model runs on user's hardware, no network
}
```

Adding a new brain is one file. The trait is the entire surface.

---

## How a user authorises a brain

The setup wizard on first launch asks: "Which AI subscriptions do you have?" Each tile is one click.

For each, Echo does the simplest thing that proves the user is signed in to the corresponding channel:

| Brain | Authorisation check |
|---|---|
| `claude` | Run `claude doctor`, expect `Signed in as <email>` |
| `codex` | Run `codex auth status`, expect `Logged in` |
| `gemini` | Run `gemini status`, expect `Authenticated` |
| `ollama` | HTTP GET `http://localhost:11434/api/tags`, expect a model list |
| `lmstudio` | HTTP GET `http://localhost:1234/v1/models`, expect a model list |

If the check fails, the wizard surfaces the exact command the provider's docs say to run. Echo never asks the user for a password, never stores a token of its own, never holds anything sensitive.

For users on locked-down corporate machines: Echo can be told to skip a brain. The user then operates on the brains they can reach.

---

## How a new brain ships

The day a new provider releases a sanctioned consumer CLI, the steps to add it to Echo:

1. Write `apps/brain/src/brains/<brain-id>.rs` implementing `Brain`.
2. Add a registry entry to `apps/brain/src/registry.rs` so the wizard offers it.
3. Add a tile to `apps/shell/src/settings/brains/<brain-id>.tsx` for the visual selection card.
4. Add a row to this document.
5. Add a section to the wiki page `Brains.md` explaining the subscription requirements.

That is the whole upgrade. Two files of code, three files of docs. No core changes.

---

## What the user sees

In the setup wizard, after picking which brain(s) they have:

> **Pick the AI subscriptions you already pay for.**
>
> Echo runs on your existing Pro plan. It will never ask you for an API key, never charge you a bill, and never sign up for a separate service on your behalf.
>
> If you have more than one, Echo picks the best for each task and respects whichever one you tell it to prefer.

After setup, in the HUD:

> BRAIN  claude (Pro)  234/300 left

If the user hits the rate cap on Claude during a long session and they have Codex configured, Echo silently falls over to Codex for the next requests until Claude's window resets.

---

## What we explicitly refuse to build

- A "free trial" mode that uses Echo's own API budget. The whole premise of the product is the user owns the cost path.
- A managed cloud tier. No SaaS on top of an open-source local tool. People can self-host the brain CLIs they already pay for.
- A "Bring your API key" mode. Even though that would unlock more capability, it would dilute the no-bills promise. If a user genuinely needs API access, they should use Claude Code directly with their key set, not Echo.
- Automatic top-ups, in-app purchases, or any kind of payment surface.

---

## Local brains: the freedom path

Echo treats Ollama and LM Studio as first-class brains from day one. A user who refuses any cloud provider can still run Echo end-to-end on their own hardware. Memory, voice, HUD, skills, all of it runs locally. The brain itself runs locally too. No subscription, no internet, no bill. Slow on a laptop, fast on a serious machine.

This is the "I just want to own my Jarvis" path. Echo respects it.

---

## The pricing promise, restated

A user with one Claude Pro subscription, an Ollama install, and Echo running:

- Pays £15-£20 a month to Anthropic for Claude Pro
- Pays £0 to anyone else
- Has full Jarvis-grade voice loop, multi-screen HUD, calendar + mail + music + home + notes + tasks + vision + memory

That is the promise. Echo never breaks it.
