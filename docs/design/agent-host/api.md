# The API and events

## Shape

JSON over HTTP, versioned under `/v1`. The contract is
`crates/bridle-api/src/types.rs`. Resource ids are either a stable id
(`a-7f3k2`) or the unique name (`w1`), wherever an agent is named.

| Method + path | Does |
|---|---|
| `GET /v1/health` | liveness + version + non-terminal agent count (no auth) |
| `GET /v1/status` | daemon, workspace, repo, principal, agent counts by state, unread human messages, latest rate limits, Claude Code version, the last CI result for the integration branch (`ci`, when `[ci] github` is on) |
| `GET /v1/agents` · `POST /v1/agents` | list all · spawn (`{name?, role, prompt?, workdir?, model?, extra_allowed_tools?, extra_env?, components?, ignore_budget?}`, where `workdir` is `{"kind":"worktree","base"?}`, `{"kind":"repo"}` or `{"kind":"path","path"}`) |
| `GET /v1/ports` · `POST /v1/ports` · `POST /v1/ports/{port}/release` | list allocations · allocate (`{pid?, label?}`; 409 when the range is exhausted) · free one (404 if not allocated) |
| `GET /v1/agents/{id}` | one agent: state, current turn's start, turns, cost, held and unacked message counts |
| `POST /v1/agents/{id}/messages` | send (`{body, kind, when, reply_to?}`) |
| `POST /v1/agents/{id}/interrupt` · `/stop` · `/resume` · `/renew` | control (`{drop_held}` · `{now}` · `{ignore_budget}` · `{ignore_budget}`); `resume`/`renew` refuse (409) under a budget hold unless `ignore_budget`; `renew` stops the agent if running and starts its replacement fresh (new session, same worktree/branch/role/model), keeping the same id and name; never refused by a budget hold (`ignore_budget` is accepted and ignored) |
| `DELETE /v1/agents/{id}` | `rm` (`?force&delete_branch`) |
| `GET /v1/agents/{id}/transcript?since=&limit=` | raw transcript lines, ascending; with `since`, lines after that line number; without it, the most recent `limit` lines (default limit 500) |
| `GET /v1/messages?to=&from=&unread=&limit=` · `POST /v1/messages` | inbox queries (`to=me` for the caller) · send to any recipient incl. `human`. A `reply_to` from a `[messages] answer_for_human` principal to a message for the human closes it (`answered_by`, `answered_reply`, `answered_line` on the `Message`; see messages.md) |
| `POST /v1/messages/{id}/read` · `/unread` | mark read · mark unread again (a read message goes back to `delivered` and the unread list; any other state is left alone; same recipient rule; 404 if unknown) |
| `GET /v1/events?since=&agent=&kind=&limit=` | the event log, oldest first (default limit 500); `agent` is an id or name; `kind` is a prefix |
| `GET /v1/events/stream?since=` | the same as **SSE**; resumable with `Last-Event-ID` (`since` wins if both are given); keep-alive every 15 s |
| `POST /v1/handovers` · `GET /v1/handovers` | write a handover note (`{body}`; `human` and `external:orchestrator` only, else 403; empty is 400) · list them, newest first |
| `GET /v1/handovers/latest` · `/{id}` | the newest note (`null` when none) · one note (`h-0007`; 404 if unknown). Any principal may read |
| `POST /v1/orchestrator/handover` | the orchestrator has written its state (human and `external:orchestrator` only, else 403): marks the handover done; the supervisor stops the session and relaunches it on its next tick ([[orchestrator-supervision]] section 6). Answers `{marked_at}`. The marker only; the note is not stored |
| `GET /v1/orchestrator/wake` | the orchestrator's long poll (`external:orchestrator` only, else 403): held until a wake condition is pending, then answers `{wakes: [{reason, text, detail}]}` and marks them delivered; answers `{wakes: []}` after 5 min. `reason` is `agent_exited`, `agent_crashed`, `agent_stalled`, `question`, `message`, `all_idle`, `usage`, `budget_hold`, `ci_failed` or `context` (a context or uptime note, section 6). While a request is open the orchestrator counts as waiting ([[orchestrator-supervision]] section 5) |
| `GET /v1/usage` | per-agent (including removed agents) and total turns, tokens, cost, busy and wall time, latest rate limits, today's `bridle statusline` snapshots |
| `GET /v1/usage/breakdown?since=&by=` | the turns ledger grouped by `role`, `model` or `agent` (default), each with turns, tokens, cost, busy time and its own cache hit ratio, plus wall time for the `agent` grouping only; `since` (RFC 3339) keeps only turns started at or after it |
| `POST /v1/statusline` | record a `bridle statusline` snapshot (`{session_id?, model?, cost_usd?, context_used_tokens?, context_max_tokens?, rate_limits: [{window, utilization?, resets_at?}]}`); rate-limit windows go through the same store path as `rate_limit_event` |
| `GET /v1/budget` | the governor's state, per-window readings and staleness, and the effective thresholds ([[../usage-and-budget#The budget governor\|the budget governor]]); the human hold, schedule override and `max_workers_override` when set |
| `POST /v1/budget/hold` · `/release` | hold spawns/resumes until `{until?}` (no `until`: until released) · lift it (human only) |
| `POST /v1/budget/max-workers` | override `[budget] max_workers` (`{max_workers?}`, none clears; human only) |
| `POST /v1/budget/override` · `/override/clear` | force a `[[budget.schedule]]` period's (or `default`'s) `five_hour` thresholds until `until` or the schedule's next change (`{period?, until?}`) · cancel it now (human only; [[../usage-and-budget#Schedule override\|schedule override]]) |
| `GET /v1/tokens` · `POST /v1/tokens` | list external tokens (name, created-at, revoked-or-not; never the token itself) · mint one (human only) |
| `DELETE /v1/tokens/{name}` | revoke `external:{name}` (human only; an agent's own token isn't revoked this way — see `rm`) |
| `GET /v1/tasks` · `POST /v1/tasks` | list (`?ready=`, `?claimed_by=` (`me`), `?top_tier=`, `?component=`) · create (`{title, kind, body?, size?, components?}`), starting `open`; emits `task.created`, and wakes the manager ([[../coordination#Waking the manager|coordination.md]]) |
| `GET /v1/tasks/{id}` · `PATCH /v1/tasks/{id}` | one task, including its body and thread · change `title`/`body`/`components`/`size` (`{title?, body?, components?, size?}`; never state) |
| `POST /v1/tasks/{id}/plan` | `open` -> `planned` |
| `POST /v1/tasks/{id}/drop` | `{reason}` (required) -> `dropped`, recorded in the thread |
| `POST /v1/tasks/{id}/reopen` | `dropped` or `integrated` -> `reopened`; any other current state is a 409 |
| `POST /v1/tasks/{id}/claim` | claims a ready task for the caller: `planned` -> `claimed`; 409 if it isn't ready (not planned, blocked, or already claimed) |
| `POST /v1/tasks/{id}/release` | releases the caller's own claim: `claimed` -> `planned`; 409 if the caller isn't the current claimant, including if it isn't claimed at all |
| `POST /v1/tasks/{id}/done` | `{commit, branch?}` -> `integrated`, recording the landing; with `branch` the commit must be on the integration branch, and the branch's agents, worktrees and branch are removed (cleanup reported in the thread); tells other workers `main moved` |
| `POST /v1/tasks/{id}/land` | `{branch?, check_cmd?}`: the integrator ([[roles-and-config]]); 409 `land_failed` leaves everything untouched; on success runs `done` and returns `{task, commit, notes}` |
| `POST /v1/tasks/{id}/summary` | `{text}`: the landing summary, kept in the task record |
| `POST /v1/tasks/{id}/impact` · `POST /v1/impact/check` | declare a task's impact (`{impact}`) · overlap-check all tasks against a spec map (`{spec_map}`), opening conflicts and returning `{overlaps, opened, probes}` ([[../impact-and-conflicts]]) |
| `POST /v1/probe` | in-memory merge probe of a branch (`{branch}` or `{target}`: task or agent) against the integration branch |
| `GET /v1/conflicts` · `POST /v1/conflicts/{id}/resolve` | conflicts opened by `impact check` · resolve one with exactly one of `{compatible}`, `{order: [a,b]}` (adds a `blocks` edge) or `{merge_into}` |
| `POST /v1/tasks/{id}/ask` · `/answer` · `/note` · `GET /v1/questions` | ask a question of a task (`{body}`) · answer it · add a thread note · list open questions |
| `GET /v1/edges` · `POST /v1/edges` · `DELETE /v1/edges` | list · add (`{from, to, kind}`) · remove (same triple as query) |
| `GET /v1/queue` · `POST /v1/queue` · `POST /v1/queue/tiers` | read · replace (`{tiers}`) · append a tier (`{tasks}`); writes are product-manager or human only |
| `POST /v1/rebuild` | rebuild the database's task tables from the state branch (human only; 409 if they aren't empty) |
| `POST /v1/restart` | restart in place (human and `external:orchestrator` only); body `{wait_secs?}` (default 600). Waits for every agent to be idle, else 409 naming the busy agents and nothing happens. Replies `{commit, agents, stop_limit_secs}` once it has decided to go; see [[docs/design/agent-host/daemon#Restart in place|restart in place]] |
| `POST /v1/shutdown` | graceful stop of all agents, then exit (human only); replies `{stop_limit_secs}`, the cap (`stop_grace` + 5 s) |

Tasks ([[docs/design/storage#The state branch|storage.md]]) are scoped, for
now, to `open`/`planned`/`claimed`/`dropped`/`integrated`/`reopened`
([[docs/design/roles-and-lifecycle#Task lifecycle|task lifecycle]]);
`in_review`/`accepted` and what drives them arrive with later tasks.

`POST /v1/agents` with a `prompt` (or a role `start_prompt`) waits briefly
after sending it for that turn's readiness before answering (a `system/init`
or an exit, whichever comes first), so a spawn failure after `claude` starts
usually comes back as a `crashed` agent in the response itself, not only
later via polling; see [[agents#Spawning|agents.md, Spawning]] for the
timeout. A spawn with no first message starts no turn and returns at once.

Every `GET` route, not just `/v1/health`, needs no bearer token: a request
with none authenticates as a synthetic `local` principal instead of 401ing
(see [[principals#Read access without a token|principals.md]]). A `GET` with
a *valid* token still authenticates normally and keeps real attribution.
Every `POST`/`PATCH`/`DELETE` route still requires one.

Errors use one shape, `{"error": {"code": "...", "message": "..."}}`, with
proper status codes. The codes are `not_found`, `conflict`, `bad_request`,
`unauthorized`, `forbidden`, `agent_not_running` (409), `shutting_down` (503,
a store call raced a graceful shutdown; retry) and `internal`.

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
| `ci.completed` (`{sha,conclusion,url}`) | GitHub Actions finished for a new integration-branch tip ([[operating-model#CI watcher\|CI watcher]]) |
| `budget.state` (`{from,to,window,utilization,resets_at,reason}`) | the governor's state changed ([[../usage-and-budget#The budget governor\|the budget governor]]) |
| `agent.interrupted` (`{dropped_held}`) · `agent.stop_requested` (`{now}`) · `agent.resumed` (`{from}`) · `agent.renewed` (`{from}`) | control requests, with the caller as actor |
| `turn.started` (`{n}`) · `turn.ended` (`{n,subtype,is_error,terminal_reason,result,usage,cost_total}`, where `cost_total` is the session's cumulative cost) | turn boundaries |
| `agent.text` (`{text}`, truncated to 2048 characters) · `tool.use` (`{name,input_summary}`) · `permission.denied` (`{denials}`) | what the agent is doing |
| `message.sent` (`{message,to}`) · `message.delivered` · `message.read` · `message.dropped` | messaging |
| `rate_limit` (`{info}`) | a `rate_limit_event` from any agent |
| `orchestrator.incident` (`{text}`) | the orchestrator supervisor needs the human (interim until incidents exist; also a `system` note to the human) |
| `disk.checked` (`{free_bytes,total_bytes,target_bytes,worktrees_bytes,data_bytes}`) | the periodic disk reading ([[operating-model#Disk monitor\|disk monitor]]) |
| `integrate.started` (`{task,branch}`) · `integrate.finished` (`{task,branch,ok,commit?,error?}`) | `bridle land` began / ended |
| `task.created` (`{task,kind,state}`) · `task.state` (`{task,to}`) · `task.edited` (`{fields}`) · `task.question_asked` / `task.question_answered` / `task.note_added` (`{task}`) | task changes |
| `edge.added` / `edge.removed` (`{from,to,kind}`) · `queue.changed` (`{tiers}`) | dependency and queue changes |
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
