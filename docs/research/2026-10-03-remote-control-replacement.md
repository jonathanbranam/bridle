# Replacing Remote Control with bridle's own way to reach agents

Oct 3, 2026 · research for ticket
[[replace-remote-control-with-bridle-s-own-way-to-reach-agents-gcvj|gcvj]]

> **Research, not a decision.** Nothing here was run against a live `claude` except reading
> `claude remote-control --help` (Claude Code 2.1.288). Claims marked **[unverified]** come from
> docs only, or from neither.

## Summary

Remote Control (RC) only works with an **interactive** `claude`; it can't attach to the headless
`claude -p` stream-json agents the daemon runs. So "every agent in the background" means bridle
carries the human's conversation itself. Most of the plumbing is already in the daemon: messages
go into a headless agent's stdin (folded into a running turn), interrupts, resume after restarts,
full transcripts, an SSE event stream. What's missing is the **human's side**: a chat view and
send box in the gateway and bridle-ui, push notifications, a terminal chat for the laptop, and
moving the orchestrator's supervision from the tmux pane to the daemon. About 6 to 9 worker
tasks, medium overall. Recommended: build the web chat in slices and keep RC on one
human-facing seat until the chat has proven itself, then retire it.

## 1. Remote Control today

From [code.claude.com/docs/en/remote-control](https://code.claude.com/docs/en/remote-control)
and [mobile](https://code.claude.com/docs/en/mobile), checked by a docs subagent on 2026-10-03:

- **Three ways in:** `claude --remote-control <name>` (what `bridle session` uses,
  `crates/bridle/src/session.rs` `claude_args`), `/remote-control` (`/rc`) in a running session,
  and `claude remote-control` server mode (`--spawn same-dir|worktree|session`, `--capacity`,
  `--continue`, `--session-id`, `--permission-mode`). `remoteControlAtStartup` turns it on for
  every session; `disableRemoteControl` turns it off (bridle keeps it on for these roles only,
  `docs/spikes/08-lean-context-findings.md`).
- **The phone and claude.ai/code can:** send prompts, watch output stream, approve or deny
  permission prompts (but not pick Auto or Bypass mode from mobile), interrupt, run some slash
  commands (`/model`, `/effort`, `/clear`, `/compact`, `/usage`, `/rename`, `/config`; not
  `/resume` or `/plugin`), attach images and files, and get **push notifications** ("when Claude
  decides" and "when actions are required").
- **Limits:**
  - Interactive sessions only. Not `-p`, not stream-json, not the Agent SDK. So bridle-hosted
    agents can't be reached by RC, which answers the first question of
    [[remote-control-for-a-hosted-orchestrator-bagg|spike bagg]] (by the docs; not tested).
  - A claude.ai subscription login (`/login`), not an API key or `setup-token`.
  - One RC session per interactive process. Outbound HTTPS only, nothing inbound.
  - An interactive session retries a lost connection indefinitely; server mode exits after about
    10 minutes offline.
- **How it has failed us:** on 2026-09-28 every RC session dropped at 19:23 and the orchestrator
  session went with it for two hours; the cause was on RC's side (`docs/context/incidents.md`).
  The 2026-09-29 loss started ticket [[the-orchestrator-stays-running-fx7x|fx7x]]. A worker's test
  once took over the orchestrator's RC name (k6b3). The NUC's `claude-rc` unit was never set up
  (`docs/context/nuc-host.md`).
- **What it costs bridle:** the orchestrator and advisors must be interactive in tmux panes, so
  bridle carries a second supervision path for them: pid and session files, a `SessionStart` hook
  for the session id, tmux pane tags, relaunch by typing into a pane, `wait-for-wake` as a
  background command, context read from statusline files
  (`docs/design/agent-host/orchestrator-supervision.md`). Headless agents get all of that from
  the daemon directly.

## 2. What bridle already has

| Piece | Where | Use for a chat |
|---|---|---|
| Headless agents over stream-json, resumable (`--resume`), renew for a fresh session | `crates/bridle-claude`, `crates/bridle-daemon/src/supervisor.rs`, `docs/design/agent-host/agents.md` | the agent itself |
| Message delivery into stdin; a message mid-turn is folded in at the next tool boundary | `docs/design/agent-host/messages.md`, spike 01 S3 | "send a prompt" |
| `POST /v1/agents/{id}/messages`, `/interrupt`, `/stop`, `/resume`, `/renew` | `docs/design/agent-host/api.md` | send, interrupt, restart |
| Full transcript (`transcript.jsonl`, `GET /v1/agents/{id}/transcript?since=`) | `agents.md` "Transcript" | the chat history |
| SSE events (`agent.text` truncated to 2048 chars, `tool.use`, `turn.*`, `message.*`), resumable by `Last-Event-ID` | `api.md` "Events" | live updates |
| `bridle agent logs --follow`; the TUI's transcript tail and inbox reply composer | `docs/design/cli.md`, `crates/bridle-tui` | terminal chat, nearly |
| Gateway: login (argon2, HttpOnly cookie), Tailscale bind, `/api/v1`, ts-rs types, fan-out to daemons, serves bridle-ui | `crates/bridle-gateway`, `docs/design/human-web-ui.md` | the web chat's server |
| bridle-ui: React + Vite SPA, login page, items list | `/Volumes/Data/work/bridle-ui-workspace/bridle-ui` | the web chat's page |
| Permission answering over MCP (`--permission-prompt-tool`), proven by spike 03, not built | `docs/spikes/03-permission-prompt-tool-findings.md`, `messages.md` "Permission prompts as questions" | approvals, if wanted |
| Handover notes, the human's prompts log (`~/.bridle/prompts.jsonl`) | `api.md`, u6w9 | handover, time tracking |

The gateway's v1 scope deliberately has **no agent control** ("a stolen session can answer and
check off, not run work", `human-web-ui.md` section 2). A chat with agents widens that on purpose.

## 3. The gaps

Sizes: built / small (a day or less of one worker) / medium (one task, a few days) / large.

| RC gives the human | Bridle today | Gap | Size |
|---|---|---|---|
| Live chat view of the transcript | Transcript and SSE in the daemon; nothing in gateway or UI | Gateway routes to list agents and page a transcript (plus an SSE or poll proxy); a chat page rendering user, assistant text and tool calls | medium |
| Token-by-token streaming | Whole messages only (no `--include-partial-messages`) | Optional; message-level updates are enough for a phone | small, skip |
| Send a prompt, see the reply | `POST /v1/agents/{id}/messages` | A guarded gateway action plus a send box; record the human as sender | small |
| Interrupt a turn | `POST /v1/agents/{id}/interrupt` | A button | small |
| Permission approvals | Headless roles run `dontAsk`/`acceptEdits` with `--permission-prompts none`; denials are events | None needed if the role's mode and allow list are set. The orchestrator and advisors run in auto mode today; whether `-p` accepts `--permission-mode auto` is **[unverified]**. Approvals from the phone would need the spike 03 MCP answerer | small (decide the mode) / large (approvals) |
| Slash commands (`/clear`, `/compact`, `/model`) | `renew` is a fresh session; model is set at spawn | Map to bridle actions (renew, renew with a model). Whether `/compact` works as a stream-json user message is **[unverified]** | small |
| Images and files | `send_user` takes text only (`crates/bridle-claude/src/process.rs`) | Upload via the gateway, save into the agent's workdir, send the path (or an image content block, **[unverified]** for stdin) | medium |
| Push notifications | None | Watch the event stream for messages and questions to `human` (and an agent's "reply to the human") and push via ntfy, as u6wk suggests | medium |
| Reach from outside home, auth | Gateway login over Tailscale, `tailscale serve` for HTTPS | The phone needs the Tailscale app; otherwise built | built |
| Several machines (dalek, the NUC) | Gateway fans out to local projects; remote actions are task 9, after br-8b98 | Already planned | medium (planned) |
| Survives restarts | Daemon resumes agents; transcripts persist; restart in place | Better than RC, which drops with the process | built |
| Context watching and handover | Built for the tmux orchestrator; headless turns report usage; `renew`; handover notes | Move the orchestrator's thresholds and forced restart onto headless agents (renew with a handover note instead of relaunching a pane) | medium |
| Wakes | `wait-for-wake` background command in the session | A headless agent is woken by a stdin message; the wake loop goes away | built (simpler) |
| Typing to it at the laptop | The tmux pane | `bridle agent chat <agent>`: transcript tail plus a send line (the TUI has both parts) | small |
| Human time tracking (u6w9) | `UserPromptSubmit` hook in `bridle session` | Count the human's messages to agents from the store instead | small |

Also to settle when an interactive role goes headless: its prompt (it assumes a pane and RC,
`crates/bridle/src/commands/orchestrator.rs`), and interactive-only tools such as
`AskUserQuestion` (a headless agent asks through messages or task questions instead).

## 4. Options

**(a) Chat in the bridle web UI through the gateway.** Gateway routes for agents, transcript,
send, interrupt and renew (human only, behind the login), a chat page in bridle-ui, ntfy pushes.
Then the orchestrator and advisors become headless roles under the daemon.
*Takes:* the gaps above, about 6 to 9 tasks. *Lost vs RC:* the Claude app (a browser tab or
home-screen web app instead), token streaming, approvals (unless built), reach without Tailscale.
*Risks:* the gateway becomes able to run work, so a stolen cookie is worth more (keep it
Tailscale-only, short sessions); the UI work is in another project; the orchestrator's
supervision gets reworked again.

**(b) Keep RC for one human-facing seat; everything else headless.** Pairs with
[[seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv|r8kv]] (a triage or "aide"
seat takes the human's conversations) and
[[make-the-orchestrator-non-interactive-headless-z485|z485]] (the orchestrator goes headless).
The human talks to the aide over RC; the aide talks to the rest by bridle messages.
*Takes:* a headless orchestrator role and supervision (medium), the aide role (small). *Lost:*
nothing the human has today, but one session still lives in a pane with RC's failure modes, and
talking to any other agent is relayed. *Risks:* the relay adds a hop and a delay.

**(c) A terminal attach command, with or without the web UI.** `bridle agent chat <agent>`
(TUI view): the transcript tail, a send line, `Ctrl-C` to interrupt. *Takes:* small. *Lost:*
the phone, unless over SSH from Termius, which the human already uses. Good as a slice of (a),
weak on its own.

**(d) Anthropic's own features.**
- **Channels** ([channels](https://code.claude.com/docs/en/channels), research preview): a
  Telegram, Discord or iMessage plugin pushes messages into a session, and the docs say it works
  with `-p`. Gives the phone, push and no Tailscale, but text only, a plugin per agent, a
  research preview, permission relay off in `-p`, and the conversation lives in the chat app, not
  in bridle. **[unverified]** with bridle's stream-json flags.
- **RC server mode** (`--spawn worktree`): Claude Code's own supervisor; competes with bridle
  owning the process (`docs/research/01-agent-runtime.md` section 2 ruled out `claude --bg` for
  the same reason).
- **Claude Code on the web / cloud sessions, Dispatch, Managed Agents:** run on Anthropic's or
  Desktop's side, not under bridle's daemon. Not a fit.
- **Agent SDK:** the same stream-json bridle already speaks; no remote layer.

## 5. Recommendation

Do **(b) now and (a) in slices**, with (c) as the laptop half of (a). Keep RC as the fallback
until the web chat has carried a mobile-only day; then drop `--remote-control` from
`bridle session`.

1. **Spike** (small, live, Haiku): in `-p` stream-json, does `--permission-mode auto` work; does
   `/compact` sent on stdin compact; does an image content block on stdin work. Answers three
   [unverified] rows.
2. **Gateway: read agents** (medium): list agents per project, page a transcript, an SSE proxy
   for `agent.*`, `turn.*` and `message.*`; ts-rs types. Read-only, so no scope change yet.
3. **Gateway and UI: chat** (medium): send, interrupt, renew, behind the login; a chat page in
   bridle-ui. The human decides the scope widening first.
4. **`bridle agent chat`** (small): the terminal half.
5. **Push** (medium): ntfy for messages and questions to `human`, and for an agent's reply in a
   chat the human started.
6. **Headless orchestrator** (medium, z485): an `orchestrator` role spawned by the daemon;
   context thresholds and forced restart become renew with a handover note; drop the pane path
   once it holds.
7. **Advisors and seats** (medium, with r8kv): named headless seats, each a chat in the UI.
8. **Multi-machine** (gateway task 9, after br-8b98) and **file and image upload** (medium).

Slices 1 to 6 make the orchestrator a background agent the human reaches from the phone. In
total, medium: roughly 6 to 9 worker tasks, plus bridle-ui work in its own project.

## Open questions for the human

1. Is a browser tab (a home-screen web app) over Tailscale acceptable in place of the Claude app?
2. May the gateway control agents (send, interrupt, renew)? It was kept out of v1 on purpose.
3. Approvals from the phone, or fixed permissions per role (auto mode if `-p` supports it)?
4. Push: ntfy (an app and a topic) or something else (rrqe, rs7p)?
5. Which goes headless first: the orchestrator (z485) or the advisors (r8kv)?
6. Keep RC on one seat for good, or retire it once the chat works?
