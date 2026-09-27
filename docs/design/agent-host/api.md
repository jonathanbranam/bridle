# The API and events

## Shape

JSON over HTTP, versioned under `/v1`. The contract is
`crates/bridle-api/src/types.rs`. Resource ids are either a stable id
(`a-7f3k2`) or the unique name (`w1`), wherever an agent is named.

| Method + path | Does |
|---|---|
| `GET /v1/health` | liveness + version (no auth) |
| `GET /v1/status` | daemon, workspace, repo, principal, agent counts by state, unread human messages, latest rate limits, Claude Code version |
| `GET /v1/agents` · `POST /v1/agents` | list all · spawn (`{name?, role, prompt?, workdir?, model?}`, where `workdir` is `{"kind":"worktree","base"?}`, `{"kind":"repo"}` or `{"kind":"path","path"}`) |
| `GET /v1/agents/{id}` | one agent: state, current turn's start, turns, cost, held and unacked message counts |
| `POST /v1/agents/{id}/messages` | send (`{body, kind, when, reply_to?}`) |
| `POST /v1/agents/{id}/interrupt` · `/stop` · `/resume` | control (`{drop_held}` · `{now}` · no body) |
| `DELETE /v1/agents/{id}` | `rm` (`?force&delete_branch`) |
| `GET /v1/agents/{id}/transcript?since=&limit=` | raw transcript lines after line `since` (default limit 500) |
| `GET /v1/messages?to=&from=&unread=&limit=` · `POST /v1/messages` | inbox queries (`to=me` for the caller) · send to any recipient incl. `human` |
| `POST /v1/messages/{id}/read` | mark read |
| `GET /v1/events?since=&agent=&kind=&limit=` | the event log, oldest first (default limit 500); `agent` is an id or name; `kind` is a prefix |
| `GET /v1/events/stream?since=` | the same as **SSE**; resumable with `Last-Event-ID` (`since` wins if both are given); keep-alive every 15 s |
| `GET /v1/usage` | per-agent (including removed agents) and total turns, tokens, cost, latest rate limits |
| `POST /v1/tokens` | mint an `external` token (human only) |
| `POST /v1/shutdown` | graceful stop of all agents, then exit (human only) |

Errors use one shape, `{"error": {"code": "...", "message": "..."}}`, with
proper status codes. The codes are `not_found`, `conflict`, `bad_request`,
`unauthorized`, `forbidden`, `agent_not_running` (409) and `internal`.

An SSE subscriber that falls more than 4096 events behind is disconnected, and
has to reconnect with the last `seq` it saw.

**Why SSE, not WebSockets.** Every client-to-daemon action is already a plain
request, so the push channel only has to go one way. SSE over plain HTTP works
with `curl -N`, resumes from an event sequence number with `Last-Event-ID`, and
needs no extra protocol in the CLI. A WebSocket endpoint can be added beside it
if a GUI needs bidirectional streaming, for example a live terminal attached to
an agent.

## Events

One append-only table with a monotonically increasing `seq`, which is also the
SSE event id:

```json
{"seq":1042,"ts":"2026-09-27T14:03:11.120Z","kind":"message.delivered",
 "actor":"human","agent":"w1","data":{"message":"m-0042"}}
```

| Kind | Emitted when |
|---|---|
| `daemon.started` / `daemon.stopping` | lifecycle |
| `agent.spawned` (`{role,model,cwd,branch}`) · `agent.state` (`{from,to}`) · `agent.exited` (`{code,signal,reason}`) · `agent.stalled` · `agent.orphans_killed` (`{count}`) · `agent.removed` · `agent.budget_exhausted` (`{cost_total}`) | agent lifecycle |
| `agent.interrupted` (`{dropped_held}`) · `agent.stop_requested` (`{now}`) · `agent.resumed` (`{from}`) | control requests, with the caller as actor |
| `turn.started` (`{n}`) · `turn.ended` (`{n,subtype,is_error,terminal_reason,result,usage,cost_total}`, where `cost_total` is the session's cumulative cost) | turn boundaries |
| `agent.text` (`{text}`, truncated to 2048 characters) · `tool.use` (`{name,input_summary}`) · `permission.denied` (`{denials}`) | what the agent is doing |
| `message.sent` (`{message,to}`) · `message.delivered` · `message.read` · `message.dropped` | messaging |
| `rate_limit` (`{info}`) | a `rate_limit_event` from any agent |
| `claude.version` (`{version, previous}`) | an agent reports a Claude Code version the daemon hasn't seen last ([[docs/design/agent-host/agents#Claude Code upgrades|upgrades]]) |

The event log **is** the provenance record: every row has an actor
([[docs/design/agent-host/principals|principals]]). Rows are kept for 30 days
(pruned daily; not configurable yet), and transcripts are the full-fidelity
record. Whether those are the right limits is open:
[[event-and-transcript-retention-34wz|retention]].

## Running the workforce remotely

Run `bridle serve --listen 0.0.0.0:7433` on the remote host (or bind to its
Tailscale address). On the laptop:

```
export BRIDLE_URL=http://host:7433        # or: ssh -L 7433:localhost:7433 host
export BRIDLE_TOKEN=<human or external token>
```

The orchestrator and TUI then have **no filesystem access to the repo**. They
read code through agents, or through later read-only endpoints (diff, file at
ref). That is the intended split: the workforce and the repo sit together, and
control is remote. Bridle has no TLS. Use SSH forwarding or a private network
such as Tailscale. How a laptop finds several remote daemons is open:
[[finding-remote-daemons-from-the-laptop-xqvg|finding remote daemons]].
