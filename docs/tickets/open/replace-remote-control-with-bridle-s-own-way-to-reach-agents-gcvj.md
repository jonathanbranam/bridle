---
id: gcvj
title: Replace Remote Control with bridle's own way to reach agents, so every agent can run in the background
kind: research
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [jttf, z485, r8kv, hrcn]
tasks: []
---

## The ask


Research (no build yet). The human, verbatim (2026-10-03, via the advisor):

> Start a research ticket and use subagents to do research on whether we could replace remote
> control with our own solution and what it would take to build so that all agents could run in
> the background.

Context: the human reaches interactive sessions (orchestrator, advisors) from the phone through
Claude Code's Remote Control, which needs a live interactive `claude` in a terminal pane. Headless
agents (managers, workers, pm) run in the background under the daemon (stream-json). If bridle
could carry the human's conversation with any agent itself (gateway / web UI / phone), every
agent could be a background agent.

## Findings

Advisor's subagent, 2026-10-03. Detail, sources and the full gap table:
[[docs/research/2026-10-03-remote-control-replacement|research: replacing Remote Control]].

- **Remote Control (RC) can't reach a headless agent.** It works only with an interactive
  `claude` (not `-p`, stream-json or the Agent SDK), needs a claude.ai login, one session per
  process ([docs](https://code.claude.com/docs/en/remote-control); by the docs, not tested). This
  answers spike bagg's first question. So a background orchestrator needs bridle to carry the
  conversation.
- **The daemon already has most of it:** stdin message delivery (folded mid-turn), interrupt,
  resume and renew, full transcripts, SSE events. The gateway has login over Tailscale and serves
  bridle-ui, but no agent routes: v1 kept agent control out on purpose.
- **Gaps** (built / small / medium / large):

  | RC gives | Size |
  |---|---|
  | Live chat view (transcript in gateway and UI) | medium |
  | Send a prompt; interrupt | small |
  | Token streaming | small, skip |
  | Permission approvals | small (fix the mode per role) / large (approve from the phone, spike 03 answerer) |
  | Slash commands (`/clear`, `/compact`, `/model`) | small (map to renew) |
  | Images and files | medium |
  | Push notifications (ntfy) | medium |
  | Reach and auth (gateway login, Tailscale) | built |
  | Several machines (gateway task 9, br-8b98) | medium, planned |
  | Survive restarts | built (better than RC) |
  | Context and handover for a headless orchestrator | medium |
  | Wakes | built (stdin; no wait-for-wake loop) |
  | Typing at the laptop (`bridle agent chat`) | small |

- **Options:** (a) chat in the web UI through the gateway, then make the orchestrator and
  advisors headless; (b) keep RC for one human-facing seat (the r8kv aide) and make the rest
  headless (z485); (c) a terminal `bridle agent chat`, as part of (a); (d) Anthropic's own:
  Channels (Telegram etc., works with `-p` per the docs, research preview, text only), RC server
  mode (competes with bridle for the process), cloud sessions (not under the daemon). (d) doesn't
  fit.
- **Recommendation:** (b) now and (a) in slices, keeping RC as the fallback until the web chat
  has carried a mobile-only day. Slices: a small live spike (`--permission-mode auto`, `/compact`
  and images in `-p`); gateway read-only agents and transcript; chat (send, interrupt, renew) in
  gateway and bridle-ui; `bridle agent chat`; ntfy pushes; headless orchestrator (z485); headless
  advisor seats (r8kv); multi-machine and uploads. **Medium overall, about 6 to 9 worker tasks**
  plus bridle-ui work.
- **For the human:** a browser tab over Tailscale instead of the Claude app? May the gateway
  control agents? Approvals from the phone, or fixed permissions per role? ntfy for push?
  Orchestrator or advisors first? Retire RC, or keep it on one seat?
