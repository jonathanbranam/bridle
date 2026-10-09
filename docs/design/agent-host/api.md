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
| `GET /v1/messages?to=&from=&unread=&limit=&id=&mark_read=` · `POST /v1/messages` | inbox queries (`to=me` for the caller; `id` keeps one message; `mark_read=true` marks what is returned read when the caller is an agent or external principal and the message is its own, ignored for the human) · send to any recipient incl. `human`. A `reply_to` from a `[messages] answer_for_human` principal to a message for the human closes it (`answered_by`, `answered_reply`, `answered_line` on the `Message`; see messages.md) |
| `POST /v1/messages/{id}/read` · `/unread` | mark read · mark unread again (the human only, 403 for anyone else; a read message goes back to `delivered` and the unread list; any other state is left alone; same recipient rule; 404 if unknown) |
| `GET /v1/events?since=&agent=&kind=&limit=` | the event log, oldest first (default limit 500); `agent` is an id or name; `kind` is a prefix |
| `GET /v1/events/stream?since=` | the same as **SSE**; resumable with `Last-Event-ID` (`since` wins if both are given); keep-alive every 15 s |
| `POST /v1/handovers` · `GET /v1/handovers` | write a handover note (`{body}`; `human` and `external:orchestrator` only, else 403; empty is 400) · list them, newest first |
| `GET /v1/handovers/latest` · `/{id}` | the newest note (`null` when none) · one note (`h-0007`; 404 if unknown). Any principal may read |
| `POST /v1/orchestrator/handover` | the orchestrator has written its state (the orchestrator's own signal, so only the human and `external:orchestrator`, else 403; the CLI sends it from `bridle handover write` when run as the orchestrator, and from the deprecated `handover done`): marks the handover done; the supervisor stops the session and relaunches it on its next tick ([[orchestrator-supervision]] section 6). Answers `{marked_at}`. The marker only; the note is not stored |
| `POST /v1/wake/stop` | `{principal?, session?}`: ends open `agent wake` waits, those waiting as `principal` (same permission check as waiting: own, or the human any) else those of `session`; answers `{stopped: n}`; neither is a 400 |
| `GET /v1/wake?principal=&timeout_secs=&session=` | any principal's long poll (`bridle agent wake`): the daemon's `wake_reasons(principal)` (`principal_wake.rs`) decides; the one reason is `message` (unread mail; a change to a task the principal watches arrives as a `task_update` message, so it wakes through the same reason; the old `task` reason is no longer emitted, and `task` / `event` stay in the type only for old clients). Answers `{reasons: [{reason, message_ids, messages, task?, event?}]}`; for a non-human caller `messages` holds the unread messages in full and they are marked read in the same store call (the human's wake leaves `messages` empty and the messages unread), or `{reasons: []}` at `timeout_secs` (cap 6900 s, 1 h 55 min, under Claude Code's 2-hour background-task limit). a planned restart or shutdown ends it with one reason `daemon_stopping` whose `text` says why (shutting down; restarting by request, by whom; restarting for an internal upgrade, to which build), which `bridle agent wake` turns into exit 6. `session` (the launcher's `BRIDLE_SESSION_PID`) keys replacement: a newer wait with the same `session` ends the open one, which answers one reason `superseded` (exit 5, nothing marked read); no `session` replaces nothing. For `principal=external:orchestrator` (the orchestrator itself only, else 403) the wait is the orchestrator's one, the same as `GET /v1/orchestrator/wake`: each queued wake (`agent_exited`, `usage`, `context`, `message`, ... as listed there) comes back as a reason with `text` and `detail` set, and defaults to a 25 min wait. `principal` is `external:NAME`, `external:advisor/NAME`, `human` or an agent; the caller must be that principal (the owner may wait for its named sessions) or the human, else 403 |
| `GET /v1/orchestrator/wake?timeout_secs=` | the orchestrator's long poll, now also served by `GET /v1/wake?principal=external:orchestrator` (`external:orchestrator` only, else 403): held until a wake condition is pending, then answers `{wakes: [{reason, text, detail}]}`, marks them delivered, and marks the message behind each `message` wake read; answers `{wakes: []}` after `timeout_secs` (default 25 min, cap 6900 s). `reason` is `agent_exited`, `agent_crashed`, `agent_stalled`, `question`, `message`, `usage`, `budget_hold`, `ci_failed`, `incident_created`, `context` (a context or uptime note, section 6), `restart`, `upgrade_draining` or `upgrade_failed` (`upgrade` is no longer raised: skipped and building are `upgrade.*` events only) ([[daemon#Restart in place|restart in place]]). While a request is open the orchestrator counts as waiting ([[orchestrator-supervision]] section 5) |
| `POST /v1/sessions` · `GET /v1/sessions` · `POST /v1/sessions/end` | register an interactive session (advisor; `bridle session advisor` does it) · list them · end one. Kept in the daemon's `sessions.json` (beside its database) across daemon restarts ([[orchestrator-supervision]] section 6) |
| `GET /v1/usage` | per-agent (including removed agents) and total turns, tokens, cost, busy and wall time, latest rate limits, today's `bridle statusline` snapshots |
| `GET /v1/usage/breakdown?since=&by=` | the turns ledger grouped by `role`, `model` or `agent` (default), each with turns, tokens, cost, busy time and its own cache hit ratio, plus wall time for the `agent` grouping only; `since` (RFC 3339) keeps only turns started at or after it |
| `GET /v1/interactions?since=` | this machine's prompt log (`~/.bridle/prompts.jsonl`, [[roles-and-config#Prompt recording]]) as a list, oldest first: `{at, event: prompt\|reply, session?, role?, machine?, project?}`; lines without `event` are prompts, bad lines are skipped, `since` (RFC 3339) keeps lines at or after it; machine-wide, so one daemon per machine is enough; the human and local (tokenless GET) readers only, else 403. The gateway's human-time reports read it ([[../human-web-ui]]) |
| `POST /v1/statusline` | record a `bridle statusline` snapshot (`{session_id?, model?, cost_usd?, context_used_tokens?, context_max_tokens?, rate_limits: [{window, utilization?, resets_at?}]}`); rate-limit windows go through the same store path as `rate_limit_event` |
| `GET /v1/budget` | the governor's state, per-window readings and staleness, and the effective thresholds ([[../usage-and-budget#The budget governor\|the budget governor]]); the human hold, schedule override and `max_workers_override` when set |
| `POST /v1/budget/hold` · `/release` | hold spawns/resumes until `{until?}` (no `until`: until released) · lift it (human only) |
| `POST /v1/budget/max-workers` | override `[budget] max_workers` (`{max_workers?}`, none clears; human only) |
| `POST /v1/budget/override` · `/override/clear` | force a `[[budget.schedule]]` period's (or `default`'s) `five_hour` thresholds until `until` or the schedule's next change (`{period?, until?}`) · cancel it now (human only; [[../usage-and-budget#Schedule override\|schedule override]]) |
| `GET /v1/tokens` · `POST /v1/tokens` | list external tokens (name, created-at, revoked-or-not; never the token itself) · mint one (human only); `name` refuses `@`, and an optional `machine` mints the visitor `external:name@machine` |
| `POST /v1/tokens/peer` | mint `peer:<machine>` (human only; `{machine}`), the token that machine's daemons forward mail with; listed and revoked (`DELETE /v1/tokens/peer:<machine>`) with the external tokens |
| `POST /v1/outbox` | mail for another daemon (`{project, to, body, kind?, when?, reply_to?}`): accepted at once into the outbox, answered `{id, project, to, state, last_error?}` after waiting up to 3 s for the first try (`state`: `delivered`, `failed` = refused for good, or `queued` = still retrying); 400 for an unknown destination or one with no peer token; later tries run in the background |
| `POST /v1/schedules` | schedule a message (`{to?, at? or cron?, tz?, body}`; `to` defaults to the caller): answered with the `Schedule` (`{id, created_by, target, body, kind, cron?, tz, next_fire_at, last_fired_at?, state, created_at}`). Agents and the human only; an agent may target only itself (403 otherwise), the human anyone. 400 for a past `at`, an unparsable time, zone or cron, or a cron that never fires; 404 for an unknown target |
| `GET /v1/schedules?all=` | the caller's schedules (every one for the human); `all` includes `done` ones |
| `DELETE /v1/schedules/{id}` | remove one; an agent's other schedules read as 404 |
| `POST /v1/hello` | `{daemon}`: a peer says it is back; this daemon flushes its outbox queue for `daemon` at once. Peer token only (403 otherwise) |
| `POST /v1/forward` | one message from another daemon's outbox (`{origin_machine, origin_daemon, origin_id, from, to, body, ...}`), peer token only (403 otherwise); answered `{message_ids}`, and a repeat of the same origin gets the same ids and delivers nothing. 404 for an unknown recipient |
| `DELETE /v1/tokens/{name}` | revoke `external:{name}` (human only; an agent's own token isn't revoked this way — see `rm`) |
| `POST /v1/tasks/{id}/watch` · `POST /v1/tasks/{id}/unwatch` | the caller starts or stops watching the task (`watchers` on the task; the creator and the claimer are added automatically); recorded in the thread and emitted as `task.watching` `{task, watching}` only when the list changed |
| `GET /v1/tasks` · `POST /v1/tasks` | list (`?ready=`, `?claimed_by=` (`me`), `?top_tier=`, `?component=`) · create (`{title, kind, body?, size?, components?}`), starting `open`, with `created_by` set to the caller (`unknown` on a task whose creator was never recorded; an absent field parses as `unknown`); emits `task.created`, and wakes the manager ([[../coordination#Waking the manager|coordination.md]]) |
| `POST /v1/tasks/submit` | `{title, kind, body}`: file an `open` task whose body starts `submitted by <principal>` (`bridle ticket submit`; visitors allowed, [[principals]]) |
| `POST /v1/tasks/{id}/priority` · `/kind` · `/skip-settle` | change the priority (`{priority}`) · the kind (`{kind}`, only while `open`) · skip the settle period (`{reason}`; human, orchestrator or project manager); each recorded in the thread, the first two as `task.priority` / `task.kind` events |
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
| `POST /v1/tasks/{id}/ask` · `/answer` · `/note` · `GET /v1/questions` | ask a question of a task (`{body}`) · answer it · add a thread note · list open questions (a worker sees only those on its own claimed task or asked by it) |
| `GET /v1/edges` · `POST /v1/edges` · `DELETE /v1/edges` | list · add (`{from, to, kind}`) · remove (same triple as query) |
| `GET /v1/queue` · `POST /v1/queue` · `POST /v1/queue/tiers` | read · replace (`{tiers}`) · append a tier (`{tasks}`); writes are project-manager, `external:orchestrator` or human only |
| `POST /v1/migrations` | record a project migration `bridle migrate` applied, as a `project.migrated` event ([[docs/design/migrations|migrations]]) |
| `POST /v1/rebuild` | rebuild the database's task tables from the state branch (human only; 409 if they aren't empty) |
| `POST /v1/restart` | restart in place (human and `external:orchestrator` only); body `{upgrade?}`. `upgrade: true` first builds the newest commit on the integration branch with green CI, in the background (the reply is `restarting: false` with a `message`: building, or nothing newer; failure wakes `upgrade_failed`) Drains (no spawns, claims or new turns) and waits, with no timeout, until no agent is mid-turn. Replies `{commit, agents, stop_limit_secs, restarting, message?}` once it has decided to go; see [[docs/design/agent-host/daemon#Restart in place|restart in place]] |
| `GET /v1/documents?q=` · `GET /v1/documents/{path}` · `PUT /v1/documents/{path}` | the daemon's own repo's documents, human token only (any other principal, or no token, is 403; a `PUT` without a token is 401). Search returns `{project, paths}`, best first; a read returns `{project, path, content, hash, branch}`; a write sends `{content, hash}` (the hash it read) and commits on the checked-out branch, replying `{project, path, hash, branch, commit}`. Errors: 404 unknown file, 400 path that isn't plain repo-relative names, 415 not text or over 2 MB, 409 stale hash, 403 detached HEAD. A save also puts the document under review if it has a pending thread. The code is the `bridle-docs` crate, shared with the gateway |
| `POST /v1/links/resolve` · `GET /v1/specs` | human only: `{targets}` to `{project, links: [{target, path}]}` (which link targets are documents); the repo's `design/specs/` files as `{project, specs}` |
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
| `daemon.started` / `daemon.stopping` | lifecycle; `stopping` data: `reason` (why: shutting down, restarting by request, restarting for an internal upgrade) and `waiters_ended` (wake long polls it ended with that reason before closing the listener) |
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
| `task.created` (`{task,kind,state}`) · `task.state` (`{task,to}`) · `task.priority` (`{task,from,to}`) · `task.kind` (`{task,from,to}`) · `task.edited` (`{fields}`) · `task.question_asked` / `task.question_answered` / `task.note_added` (`{task}`) | task changes |
| `edge.added` / `edge.removed` (`{from,to,kind}`) · `queue.changed` (`{tiers}`) | dependency and queue changes. A queue change also sends a `system` note ("queue updated: re-read `bridle queue`…") to the running manager, else `external:orchestrator`: trailing-edge debounce of 30 s, none when a manager made the change |
| `upgrade.skipped` · `upgrade.building` · `upgrade.built` · `upgrade.draining` · `upgrade.failed` · `upgrade.rolled_back` (`{commit, ...}`) | self-upgrade steps ([[docs/design/agent-host/daemon#Upgrade|upgrade]]) |
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
