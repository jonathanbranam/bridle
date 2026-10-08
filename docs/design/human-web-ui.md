# The human web UI: the bridle gateway

> **Status (checked 2026-10-03):** Built and in use: `bridle gateway` (`crates/bridle-gateway`), build tasks 1–8 below: config and serve, discovery and fan-out, listing, local actions, retract handling, login, `/api/v1` with ts-rs types (`crates/bridle-gateway/bindings/`), serving `~/.bridle/ui/` with the version check; `bridle-ui` is its own project under bridle (`/Volumes/Data/work/bridle-ui-workspace`) · task 9 (multi-machine: listing and actions on remote projects with `[human.<machine>]` tokens), task 10 (service install)

Ticket: [[docs/tickets/open/a-web-ui-for-the-human-my-to-dos-and-decisions-to-run-throug-essy|essy]]
("Option F" and "Gateway v1: the human's answers", decided by the human, 2026-10-02). This
replaces the earlier option C (a Rust-rendered page, `bridle ui`).

The shape: a **`bridle gateway`** process in bridle's workspace serves one versioned API; a
separate **TypeScript UI** (`bridle-ui`, a later project) is a client of it. The browser never
holds a bridle token and never talks to a daemon.

## 1. The gateway

- **A subcommand of the `bridle` binary**, its own per-machine process (launchd or systemd, like
  the daemons), not inside any daemon. Daemons are per project and restart often; the gateway
  spans projects and outlives those restarts. It must never be on a daemon's start-up path, and
  a daemon needs no change to be reachable by it.
- **Discovery and fan-out:** it finds every defined daemon the way `bridle --project` does
  (`~/.bridle/daemons/<project>.json`, `[projects]` and `[machines]` in `~/.bridle/config.toml`,
  `bridle_api::machines`), queries them concurrently with a short timeout, and merges the
  results. A daemon or project that doesn't answer is **reported as unreachable** in the
  response (a sleeping laptop's projects, say), not as an error.
- **Tokens:** it holds the human's token per machine and acts with it. Nothing records that an
  action came through the gateway. Gateways don't talk to each other; one gateway reaches
  whatever its machine's config and credentials allow.
- **Contract with the daemons:** it uses `bridle-api` (as the CLI does), so a daemon API change
  breaks the gateway's build in `just check`. The daemons' `/v1` API stays internal.
- **No new daemon endpoints** for v1: to-dos are `GET /v1/tasks?claimed_by=human`, task
  questions are on the tasks, and acting is `POST /v1/tasks/{id}/done`, `/drop`, `/answer`.

### Running it detached, and staying current

The same pattern as the daemon ([[docs/design/agent-host/daemon#Running it|daemon, running it]]),
on dalek and on every client machine alike.

- **`bridle gateway --detach`** re-executes `bridle gateway` in a new process group (the
  daemon's spawn code, shared, not copied) with stdout and stderr appended to
  `~/.bridle/gateway.log`, then waits up to 60 s for `GET /api/v1/health` at the configured
  `bind` to answer. It prints the URL and pid and exits 0; if the gateway is still starting
  then, it says so, leaves it running and exits 0. If the gateway exits early it fails with the
  tail of the log. A bind with port 0 can't be probed, so `--detach` refuses it.
- **A second start is refused.** Before spawning, `--detach` probes `bind`'s health; if
  something answers it exits 1 naming the address. The foreground `bridle gateway` has the
  bind error as its guard. The port is the lock; the pid file (below) is for stop and status.
- **Pid file** (76td). The gateway process itself (foreground, or the `--detach` child, not the
  parent) writes `~/.bridle/gateway.pid` (decimal pid and a newline, beside `gateway.log`) once
  it is listening, and removes it on a clean exit and on SIGTERM or SIGINT. A stale file (pid
  not alive) is ignored and overwritten by the next start. A self re-exec keeps the pid, so
  the file stays right.
- **`bridle gateway status`** prints `running pid <pid> <url> build <build>` (the URL is
  `[gateway] public_url`, else `http://<bind>`; the build is what the gateway's own health
  endpoint reports: crate version, binary size and modified time, compared with the installed
  binary's to add `stale binary`), or `not running` (plus `stale pid file removed`). Exit 0
  running, 1 not. `--json` too.
- **`bridle gateway stop`** reads the pid file, checks with `ps -p <pid>` (that one pid, never a
  search by name or port) that its command line contains `bridle` and `gateway`, sends SIGTERM,
  and waits up to 10 s for it to exit (`stopped`). Not a gateway: it says so, signals and removes
  nothing, exit 1. Still alive after 10 s: an error naming the pid, exit 1, no SIGKILL. No pid
  file: `not running`.
- **`bridle gateway restart`** is stop (not running is fine; a failed stop means no start), then
  `bridle gateway --detach`, then the status line. Auto-restart is not built (the human).
- **It keeps itself current.** The running gateway checks every 30 s whether the file at
  its own executable path changed (modified time or size) and, if so, re-executes that path
  with the same arguments. The pid, process group and log handles are kept, so a detached
  gateway stays detached and a launchd or systemd unit sees no exit. The daemon's upgrade
  (`cargo install` over the same path) and a client machine's release update both replace that
  file, so one rule covers both. If the new binary fails to start (bad config, say) the exec
  has already happened and the process exits non-zero, which a service unit restarts; the log
  says why. Rejected: a daemon hook that restarts the gateway after an upgrade. The gateway
  must not depend on a daemon (a client machine may have none, and a machine has many), and
  a daemon restart then missing the gateway is exactly the failure to avoid.
- **Config** (`[gateway]` in `~/.bridle/config.toml`, extending
  [[a-gateway-section-in-bridle-config-toml-stops-every-daemon-f-jmpf|jmpf]]): `bind` (exists;
  default `127.0.0.1:7878`) and `enabled` (new; default `true`). With `enabled = false`,
  `bridle gateway` (foreground or `--detach`) prints that and exits 0 without starting, so
  a machine that shouldn't run one can say so and a launcher can stay installed. Both keys
  have defaults, so existing configs keep working unchanged.
- `bridle gateway install` stays for those who want launchd or systemd to start it at login.
  Its unit runs the foreground command; that is unchanged.

## 2. v1 scope

The human's **to-dos and task questions** (decisions), grouped by project, decisions first,
each list high priority then oldest. Actions: check off a to-do, answer a task question, decline
a to-do with a reason, and reply to any task (`.../tasks/{id}/reply`, a note authored `human` on the
thread, any task state). **Retracted items are hidden** from the lists; the withdrawal and its
reason stay on the record as the audit trail (the asker withdraws, "To-do withdrawn by ...").
Out for now: questions sent as messages to `human` (they can't be retracted) and the inbox; no
events stream and no agent control, with one exception (rk7k): the human may **send a message to
one agent or the orchestrator** of a project (`POST .../messages`, recipients from
`GET .../recipients`: that project's running agents plus the orchestrator). It is sent with the
human's token through the daemon's ordinary send endpoint, so the sender is `human` and the
message is in the audit trail; an optional task id threads it on that task. Text is non-empty and
capped, and any other recipient is refused (422). Still no start, stop, kill or events: a stolen
session can message, not run work directly. One project per request; a cross-project picker is
open. The gateway may also expose **read-only views** of tasks
(s6cj, below), and of agents and status (br-7sd9); a read changes nothing. It exposes only the v1
actions as writes and refuses the rest, so a stolen session can answer and check off, not run work.

## 3. API, login, UI

- **One versioned API** (`/api/v1/...`), at most one previous version served beside it.
  TypeScript types are **generated from the Rust types by ts-rs** and committed or emitted by a
  `just` target for `bridle-ui` to consume. (utoipa plus openapi-typescript only if a full
  endpoint spec becomes worth it.)
- **Login:** username and password; the `argon2` hash lives in the machine config. Generate it with `bridle gateway hash-password` (reads from stdin or a terminal prompt, asks for confirmation on a terminal). A successful
  login sets an `HttpOnly`, `SameSite=Strict` session cookie. Reached over Tailscale (bind the
  Tailscale or loopback address, never 0.0.0.0); `tailscale serve` can add HTTPS.
- **The UI** is its own TypeScript repo, `bridle-ui`, beside `bridle/`. Its build output is
  installed into a folder the gateway serves, **`~/.bridle/ui/`**, so page and API share an
  origin. The build records the API version it targets; the gateway warns or refuses on a
  mismatch (`ui_version_mismatch`, default warn). The UI files are served without a session, since
  the page must load to show its login form; every API route stays guarded. Not compiled into the `bridle` binary (no Node in bridle's build). In development
  the UI's dev server proxies API calls to the gateway.

**Documents (br-5paw, ticket x8jt; a narrow piece of v8kn, built).** For the UI's document view
(`documents.rs`), behind the session like the rest: `GET /api/v1/projects/{project}/documents/{path}`
returns `Document` (`content`, `hash` = SHA-256 hex of it, and the `branch` checked out in the
project's working tree); `PUT` the same URL with `DocumentWrite` (`content` = the whole file,
`hash` = the hash it was read at) writes the file and commits just that file as
`review: human comments on <path>`, returning `DocumentSaved` (new `hash`, `branch`, `commit`).
A write whose `hash` no longer matches the file is a 409 and changes nothing; unchanged content
commits nothing. Refused: a path with `..`, an absolute path, `.git`, or one that resolves
(symlinks included) outside the repo (400); a missing file (404, a write never creates one);
a non-text file (NUL or invalid UTF-8) or over 2 MB (415); and any write while the working
tree is on a detached HEAD (403): the gateway commits on whatever branch is checked out in the
project's working tree, including `main`. This machine's
projects only (the repo comes from the daemon's registry entry).

**Document search and auto-review (br-jrm2, ticket jrm2).** `GET /api/v1/projects/{project}/documents?q=`
returns `DocumentMatches {project, paths}`: up to 30 `.md` paths under `docs/` containing `q`
(case-insensitive), best first: a ticket whose ID is exactly `q` (a bare `x8jt` finds
`docs/tickets/*/<slug>-x8jt.md`), then open tickets, then open spikes, then the rest. An empty `q`
lists the open tickets. Only `docs/` for now; widening it is later. After a `PUT` the gateway asks
the daemon to put the path under review if the saved text has a pending thread (`POST
/v1/review/add` with `only_if_pending`, the human's token, as for review now), so a comment saved
from the UI needs no `bridle review add`. Adding a path already under review changes nothing, a
daemon that can't be reached is logged and doesn't fail the save, and a comment added by hand
still needs `bridle review add` (nothing scans the repo for them; whether anything should is
undecided). The UI side (project dropdown, search box, margin layout) is bridle-ui's.

**Link resolution (br-bnhn, ticket bnhn).** `POST /api/v1/projects/{project}/links/resolve` with
`LinkResolveRequest {targets}` returns `ResolvedLinks {project, links}`, one `{target, path}` per
target in order, `path` null when nothing matches. A target with a `/` is a path under `docs/`
(`docs/design/gates` or `docs/design/cli.md`; `.md` is tried if the target as written is no
file); one without is a ticket stem, looked up in `docs/tickets/open/` then `resolved/` (tickets
move); failing that, a bare ticket ID (4 characters) or a task ID `<prefix>-<id>` whose `<id>`
is a ticket ID (a ticket's first task takes its id) resolves to the ticket file ending `-<id>.md`
(br-a3yd). Other task IDs resolve to null (no task view yet); cross-project IDs wait on question
j28f. Targets are checked on disk with the same rules as a document read, so `..`, absolute
paths, symlinks out of the repo and anything outside `docs/` resolve to null, never an error.
Bare names like `README` are not resolved (undecided). The UI turns wiki links and paths into
links with the answer; rendering is bridle-ui's.

**Specs (br-75zr, ticket 75zr).** Read-only. A document read already serves `design/specs/**`
(the read route accepts any text file in the repo, `.git` aside); search stays `docs/` only.
`GET /api/v1/projects/{project}/specs` (`specs.rs`) returns `ProjectSpecs {project, specs}`:
one `SpecFile {path, capability, title, requirements, diagnostics}` per `.md` under
`design/specs/`, parsed with `bridle-spec`; each requirement has `id`, `heading`, `protected`,
`line` and `scenarios` (`id`, `heading`, `executable`, `line`). A file that doesn't parse has
`diagnostics` (`line`, `column`, `message`) and no requirements; it is not an error. No
`design/specs/` is an empty list. `links/resolve` also takes a requirement or scenario id
(`r-xxxx`, `s-xxxx`, tried before the task-id rule) and answers `design/specs/<file>.md#<id>`
(the heading's explicit `{#id}` anchor), and a capability name (file stem, tried after tickets)
and answers the file; unknown ids are null. Linking tasks to specs is not built: the UI links ids
it finds in task text through `links/resolve`. The `PUT` route is unchanged and can still edit a
spec file like any document; the spec view itself never writes.

**Review now (br-qttb, ticket x8jt).** `POST /api/v1/projects/{project}/review` with
`ReviewRequest {path, resend}` (`resend` defaults to false) asks the project's daemon to send the
document's unsent comment threads to its agent at once, like `bridle review now`; returns
`ReviewResult {project, path, agent, threads}`. `threads: 0` means nothing was unsent (the UI can
say so). The path must be one under review (`bridle review add`), else 400 with the daemon's
message; the daemon is reached with the human's token like the task actions. Sent threads are
marked in the file (`[sent YYYY-MM-DD HH:MM EDT]`, ASCII; the daemon turns it into `[read ...]` once the agent has read the message, and gives a hand-typed thread its `c<n>` ID), so re-reading the document shows what went. The UI
button is bridle-ui's ui-c39e.
The commit uses the repo's git identity.

**Tasks, read-only (br-s6cj, ticket s6cj; `tasks.rs`).** For the UI's Tasks page; the daemon is
read with the gateway's per-project credentials, as for review. `GET /api/v1/projects/{project}/tasks?state=open|closed|all`
returns `TaskList {project, tasks}`, most recently updated first; `open` (the default) is every
task not integrated or dropped, `closed` is integrated or dropped. Each `TaskSummary` has `id`,
`title`, `kind`, `state`, `priority`, `claimed_by`, `agent` (`{name, role}` when `claimed_by` is
`agent:<name>` and the daemon knows it, else null) and `updated`. `GET .../tasks/{id}` returns
`TaskDetail` for a task in any state (so closed tasks open from links): the summary plus `body`,
`thread`, `watchers`, `branch`, and its `blocks` and `blocked_by` task ids (from `/v1/edges`; other
edge kinds aren't shown). An unknown project is 404, an unknown task the daemon's 404. Not built:
live updates (events; static first) and tickets (until their IDs settle, j28f).

**System and agents, read-only (br-7sd9, ticket 7sd9; `system.rs`).** For the UI's System page.
`GET /api/v1/projects/{project}/system` returns `SystemView {project, reachable, error, status}`:
`status` is the daemon's `/v1/status` reduced to pid, version, start time, Claude version,
`budget_state`, rate limits, CI, incidents, sessions, agent counts by state, unread human
messages and `upgrade_waiting`. A daemon that is down (or whose address or token can't be found)
is a 200 with `reachable: false` and `error`, so the UI says "unreachable"; an unknown project is
404. `GET .../agents` returns `AgentList {project, agents}`: name, role, state, `stopped` (not
running: stopped, exited, crashed or lost; those are included so the UI can hide them), model,
`task` (the open task claimed by `agent:<name>`), branch, context tokens, cost, turns,
`exit_reason` and `updated`; running agents first. **Dropped, by building the wire types from an
allow-list:** the principal; the daemon's workspace, repo and URL; an agent's session id, pid,
cwd, worktree, created_by and message counts; a session's pid, pane and Claude session id. No
token or credential path is in the daemon's status or agent records. Not built: servers and ports
(`/v1/ports`; later, once this is in use) and live updates.

**Human time (ticket u6w9; built: types, collection, interval math, handlers).** The wire types of `/api/v1/interactions/*` live in
`crates/bridle-gateway/src/interactions.rs` and are exported to `bindings/` like the rest:
`InteractionReport` (`report`: totals per group per day or week, plus the human's total),
`DayReport` (`day`: per-session intervals, overlaps, peak concurrency, minutes at 1, 2 and 3+),
`HoursReport` (`hours`: minutes per hour of day, averaged over the matching days),
`IntervalsReport` (raw intervals). Each carries `unreachable` machines. Times are RFC 3339 UTC
strings; days split at US Eastern midnight.

*Collection* (`collect.rs`, started by `bridle gateway`): every 5 minutes the gateway reads each
machine's prompt log (`GET /v1/interactions?since=`, one answering daemon per machine) and each
project's messages `from=human` (a message is a point prompt in a session `message:<recipient>`,
agent = the recipient), and merges them into its own append-only store,
`<bridle home>/gateway-interactions.jsonl`, deduped by (machine, session, time, event) or
(machine, project, message id). A daemon that doesn't answer is reported in the poll's
`unreachable`, not fatal; its data catches up when it is back.

*Intervals* (`intervals.rs`, pure functions), per session: a prompt counts through the agent's
turn (waiting is the human's attention) and on to the next prompt if that comes within `gap` of
the reply finishing, else the run ends at reply end + `tail`. With no reply recorded, a prompt
counts to the next prompt if within `gap` of it, else `tail`. A run's first prompt gets `lead`
before it. Human time is the union of all sessions' intervals; concurrency is how many sessions
cover a moment. `[interactions] gap`, `tail`, `lead` in `config.toml` (defaults 10m, 2m, 1m).

*Handlers* (`report.rs`, all behind the session login like the rest of `/api/v1`; dates are
`YYYY-MM-DD` in US Eastern, `to` inclusive, a range at most 400 days; a bad or missing parameter
is a 400 with `{"error"}`):
`GET /interactions/report?from&to&group=project|agent|machine&bucket=day|week` (defaults project
and day; weeks start Monday and their `start` is that Monday; intervals are cut at Eastern
midnight, so a run over midnight counts on both days; groups sorted by minutes);
`GET /interactions/day?date=` (that day's intervals per session, overlaps, peak and minutes at
1, 2 and 3+ at once);
`GET /interactions/hours?from&to&days=weekday|weekend|mon,tue,...` (minutes of human time per
Eastern hour of day, averaged over the matching days, default all days);
`GET /interactions/intervals?from&to` (raw intervals touching the range, uncut). Eastern uses
the US daylight rule (second Sunday of March to first Sunday of November) in code, no timezone
database. `unreachable` is who the last poll couldn't read.

## 4. Multi-machine

The human runs one gateway on one machine and manages everything from there. For a project on
another machine (`[projects]` + `[machines]`) the gateway uses the human's token from
`[human.<machine>]` in `~/.bridle/credentials.toml` (br-8b98), for listing and for actions alike
(`actions::resolve`). A local project keeps the workspace's human token. A machine that doesn't
answer is listed as unreachable; one with no `[human.<machine>]` entry for the project is reported
with an error naming that section. A project on a random port needs a `[projects]` entry with a
fixed port to be reachable from another machine.

## 5. Build tasks, in order

Each is one branch and one worker. None touches the daemon. Tasks 1–10 are built; 11 is
planned.

1. **Skeleton, config, serve**: `bridle gateway` subcommand, a `crates/bridle-gateway` library
   (axum), the optional gateway config section, bind address, `GET /api/v1/health`. Tests: the
   server starts, answers, and rejects a bad bind.
2. **Discovery and fan-out**: reuse registry, `[projects]`, `[machines]`; concurrent queries
   with a timeout; unreachable daemons reported. Tests with fake daemons: one up, one down, one
   slow.
3. **Listing across projects**: to-dos and task questions, grouped and ordered per section 2.
   Tests: ordering, grouping, two projects merged.
4. **Actions**: check off, decline, answer, with the human's token (local machine). Tests: each
   action reaches the right daemon; a missing token is a clear error.
5. **Retract handling**: hide withdrawn items, keep them on the record. Tests: a withdrawn
   to-do and a withdrawn question don't list; the audit trail is still on the daemon's record.
6. **Login and session**: argon2 config, login and logout, cookie, every other route behind it.
   Tests: no cookie is refused, a wrong password is refused, a cookie of the right flags.
7. **Versioned API and ts-rs types**: move routes under `/api/v1`, the types derive `TS`, a
   `just` target emits them; a test that fails if the emitted files are stale.
8. **Static UI folder and version check**: serve `~/.bridle/ui/`, compare the build's recorded
   API version, warn or refuse. Tests: serves a file, a missing folder, a mismatched version.
9. **Multi-machine** (built, br-7172): per-machine human tokens for remote daemons, remote
   actions. Tests: a remote fake daemon with its own token.
10. **Service install** (built): `bridle gateway install` writes the launchd plist / systemd user unit (`dev.bridle.gateway`, `bridle-gateway.service`), restart on failure only, log `~/.bridle/gateway.log`; loading it is the operator's step.
11. **Detach and self-restart** (bek3): `bridle gateway --detach`, `[gateway] enabled`, the gateway
    re-executing itself when its binary changes. Tests: detach starts, logs and answers health; a
    second start is refused; a changed binary is re-executed (with a fake).

`bridle-ui` itself (and an install script) is a separate project, not part of these tasks.

## 6. Migration

No project file, schema or daemon change. The only addition is new **optional machine config**
(the gateway section with the login, bind address and `enabled`, default true, in `~/.bridle/config.toml`); a machine
that doesn't run a gateway never sees it.
