---
id: v8uu
title: "Seeing what background agents do: their stream, the messages between them, steering and status updates"
kind: research
opened: 2026-10-07
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [gcvj, g6v4, hrcn, y496, 34wz]
tasks: []
---

## The ask

The human, 2026-10-06 ~11:40 PM ET, via aide:

"Research question, investigate this, find the answer, write a ticket, then send me a todo to review the ticket:

If I wanted to see the tokens, thinking, tools calls, etc for a background agent using our json streaming, is that possible and how can we enable that in the UI as a stream of what tokens are going in and out of the agent. I want some visibility into what the agents are doing.

Also, could I watch the stream of messages that are being passed between agents? Can I send agents a message while they are running? Background agents I mean. And how quickly will they respond? Could we have them give status updates about their progress and write summaries of what they are doing?

Theses are just research questions to investigate so I can learn more and consider future features. thx."

Findings below are from the code at f849402e and the docs, 2026-10-07. No build is asked for yet: the human reviews this, then decides what (if anything) becomes features.

## 1. An agent's stream: tokens, thinking, tool calls

**What Claude Code gives us** (docs/spikes/01-stream-json-findings.md): one `assistant` event per content block (text, tool_use); `user` events with tool results; a `result` per turn with token usage and cost. **Thinking text is not available**: only its signature and an estimated token count. Token-by-token streaming (`--include-partial-messages`) is not used or tested; we get whole blocks, so "live" means block by block (each paragraph or tool call as it completes), not word by word.

**What bridle keeps (built):**
- Every raw line in and out of each agent: `.bridle/agents/<id>/transcript.jsonl` (kept after `agent rm`; retention is ticket 34wz).
- Parsed events in the database (30 days): agent text (cut at 2048 chars), tool calls (name + input summary, no output), turn start/end with usage and cost.

**Where you can see it today:**
- CLI: `bridle agent logs <agent> --follow` (text, `-> Tool(input)`, turn results; `--raw` for everything, including tool results). `bridle events --follow` for the live event stream.
- TUI (`bridle tui`): live event tail and a per-agent transcript tail.
- API: `GET /v1/agents/{id}/transcript`, SSE `GET /v1/events/stream`.
- **Web UI: not at all.** The gateway has only a read-only agent list; no transcript, no events. docs/design/human-web-ui.md keeps a live stream out of scope on purpose.

**To get it in the web UI:** gateway routes to page an agent's transcript and to pass on the event stream (SSE), plus an agent page in bridle-ui rendering text, tool calls and results, and turn usage/cost. This is mostly planned already in ticket gcvj (replace Remote Control: list agents, page a transcript, a chat page), sized medium, not built. Optional later: token-level streaming via `--include-partial-messages` (untested; rated "small, skip" in docs/research/2026-10-03-remote-control-replacement.md).

## 2. Watching messages between agents

- The daemon's API can already list **all** messages (`GET /v1/messages` with `from`/`to` filters), and the human's token can read them. Events `message.sent/delivered/read` are live on `bridle events --follow`, but without the body.
- **No view exists**: `bridle inbox` and the TUI show only your own messages; the web UI only sends. No ticket for this yet.
- Smallest step: a CLI `bridle messages [--follow] [--from] [--to]` and a web UI "messages" page over the existing API.

## 3. Sending a message to a running background agent

**Yes, built.** `bridle send <agent> "..."` (or the web UI's send box) writes it to the agent's input as a new user message:
- `--when now` (default): the agent sees it at its **next tool boundary**, mid-turn.
- `--when idle`: held until its current turn ends.
- `bridle interrupt <agent>` stops the current turn (not in the web UI).

## 4. How quickly they respond

Measured in spike 01: a message sent while a tool ran was seen when that tool finished (sent at 2 s, seen at 14 s, after a 12 s tool). So the delay is "until the current tool call ends": usually seconds, longer behind a slow build or test run. An idle agent picks it up at once. An interrupt takes effect in about 1 s. Unverified: an agent generating text with no tool calls probably sees it only at turn end (spike akjw, open).

## 5. Status updates and summaries

**By convention today, not automatic:** workers are told to post progress comments on their task as they go, and must write a summary (`bridle task summary`) and a "done:" message when finished; managers reject work without a summary. These show on the task's thread in the web UI, and watchers of a task get a one-line message per change.

**Not built:** periodic or automatic status updates or summaries. Related: g6v4 (how agents send the human updates vs. things to act on), hrcn (scheduled/recurring messages), y496 (TUI: seeing the work in progress).

## Possible features (for the human to choose from)

1. **Agent live view in the web UI** (per agent: text, tool calls, results, tokens/cost per turn). Builds on gcvj. Biggest visibility win.
2. **Messages view** (CLI + web UI) of all agent-to-agent traffic. Small; the API exists.
3. **Interrupt in the web UI** next to send. Small.
4. **Periodic progress summaries**: e.g. every N minutes or per turn, a one-line "what I'm doing" on the agent, cheap if taken from the transcript (last text block) rather than asking the model. Needs design; ties to g6v4.
5. **Token-level streaming** (`--include-partial-messages`): only if block-by-block feels too coarse.
