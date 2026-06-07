# HUD Design

The visible part of Echo. Glance, do not stop. Multi-monitor aware. Iron Man references stay tasteful.

## Principles

1. **Sub-second comprehension.** Every panel communicates in under a second of attention. No paragraphs.
2. **Glass and ink.** Translucent surfaces, near-monochrome ink, one accent (bioluminescent teal `#34d399`). Not blue glow everywhere.
3. **Always-on-top, never in the way.** Default top-right corner of the primary monitor, 380px wide. Click-through when idle. Hover or wake-word to interact.
4. **Multi-monitor first.** Detection at startup and on hot-plug. Roles per display.
5. **State is visible.** Listening, thinking, speaking, idle, offline. The user always knows what Echo is doing.

## Surfaces

### Primary card

```
+-------------------------------------------+
| [echo]    23 Aug, 14:32    HEMEL    o     |
+-------------------------------------------+
|                                           |
|   live transcript scrolls here            |
|   "what's on today"                       |
|   <- reply streams in here                |
|                                           |
+-------------------------------------------+
|  NEXT      Standup, 5 min                 |
|  TODAY     12C clear, rain at 18:00       |
|  INBOX     3 unread                       |
|  SYS       CPU 21%  MEM 8.2G  NET 41kb    |
|  BRAIN     claude (Pro)  234/300          |
+-------------------------------------------+
```

Width 380px, height adjusts to content. Glass card with subtle border and drop shadow.

### Dashboard (secondary monitor)

Full-window layout. Configurable panels: live metrics, today's calendar, the proactive watch queue, the skill catalogue with green/yellow/red status dots, the latest five facts written to memory.

### Transcript wall (secondary monitor)

A single big-text rolling transcript. Useful for noisy environments, for showing a guest what Echo is doing, or for screen-recording demos.

### Ambient (peripheral monitor)

Rotates every 12 seconds through a small set of glance panels: weather + forecast, top three calendar events for the day, top three news items, latest fitness day, finance summary.

## States and animations

| State | Visual |
|---|---|
| Idle | Faint breathing pulse on the wake-word indicator (4 seconds in, 4 seconds out, alpha 0.4 to 0.8) |
| Listening | Bright teal ring pulses outward (every 1.4s), waveform shimmer at the bottom |
| Thinking | Clockwise rotating sweep around the wake-word indicator (1s rotation) |
| Speaking | Live waveform of the current sentence at the bottom of the card |
| Skill running | A small chip with the skill name appears in the bottom row |
| Tool call | A tiny dot trail from the brain row to the skill chip |
| Brain offline | Brain row turns amber, name in italic, "(retrying)" trailing |
| Error | Brief red flash on the affected row, error text below for 5 seconds |

## Typography

| Element | Font |
|---|---|
| Wake-word "echo" | SF Pro Display / Inter, 11px, 0.18em letter-spacing, lowercase |
| Date / location | SF Pro Display / Inter, 10px, all caps, 0.12em |
| Transcript user line | Geist Sans, 14px, weight 500, ink |
| Transcript reply | Geist Sans, 14px, weight 400, slate-700 |
| Section labels (NEXT, TODAY, etc) | SF Mono / JetBrains Mono, 10px, all caps, 0.16em, ink |
| Section values | Geist Sans, 12px, weight 500 |

## Colour palette

| Token | Hex |
|---|---|
| ink | `#0b1220` |
| slate-700 | `#334155` |
| slate-500 | `#64748b` |
| accent | `#34d399` (bioluminescent teal) |
| accent dim | `rgba(52, 211, 153, 0.35)` |
| amber warn | `#fbbf24` |
| red alert | `#f43f5e` |
| glass bg | `rgba(15, 23, 42, 0.62)` (dark) / `rgba(248, 250, 252, 0.78)` (light) |
| glass border | `rgba(255, 255, 255, 0.12)` (dark) / `rgba(15, 23, 42, 0.08)` (light) |

Adaptive to the system colour scheme. Same card, two looks.

## Hot keys

| Key | Action |
|---|---|
| `Ctrl/Cmd + Space` | Toggle command bar in the HUD |
| `Ctrl/Cmd + Shift + Space` | Mute / unmute mic |
| `Ctrl/Cmd + .` | Stop current TTS |
| `Ctrl/Cmd + ,` | Open settings |

## Mic and screen indicators

A hardware-style mic indicator in the wake-word row. Off = grey dot. Listening = pulsing teal. Muted (manual) = red dot, never bypassable by skills.

A small "screen viewing" indicator appears when a skill is capturing the screen. Same red-dot semantics: visible whenever it is happening, never silently.

## Multi-monitor configuration

The user assigns each display a role in the settings panel:

```yaml
displays:
  "Built-in Retina":
    role: primary
    position: top_right
  "DELL P2723D":
    role: dashboard
  "LG UltraFine":
    role: ambient
```

The Tauri shell creates one window per role on the matching display. Roles can be reassigned on the fly without restart.

## Accessibility

- Keyboard-only command bar (`Ctrl/Cmd+Space`). Type the prompt, see and hear the reply.
- Subtitle mode: live captions in a bigger font, configurable size, high-contrast border.
- Reduce motion preference honoured: pulses become discrete state changes, no animation.
- All colour pairs meet WCAG AA contrast.

## Wording

- Echo speaks in the first person.
- Default voice character: calm, dry, never sycophantic. "Yes" not "Absolutely, I'd be delighted to help with that!"
- Errors are honest: "I couldn't reach Google Calendar, would you like me to retry."
- Confirmations are short: "Done." rather than "I have successfully completed your request."

## Anti-patterns

- No bouncing icons or attention-grabbing animations when idle.
- No notifications that survive a click away.
- No "Echo Assistant" branding in the corner. The product is just "echo" in lowercase.
- No third-party logos in the HUD even when the relevant skill is in use.
- No emoji in any user-facing text (a hard rule across all Sarma's projects).
