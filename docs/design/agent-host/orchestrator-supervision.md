# Orchestrator supervision

> **Status (checked 2026-10-03):** Built and in use: slices 1a, 1b, 2 and 3 (liveness, relaunch and backoff, wake conditions and `wait-for-wake`, context and uptime notes, forced restart, `handover done`, the handover note as a record), advisor session registration, `session.context` steps with warnings, `session keep` and the forced restart at the hard limit · Planned: `bridle orchestrator pause`, wakes for advisors, automatic crash restart for advisors (tabled), percentage thresholds, filing supervisor incidents as `incident` tasks (it still sends a `system` note plus an `orchestrator.incident` event)

Design for ticket [[the-orchestrator-stays-running-fx7x|fx7x]]; signals verified in
[[docs/spikes/07-orchestrator-supervision-findings|spike 07]]. **Slice 1a built** (br-a424): liveness, relaunch and crash-loop backoff (sections 1 to 4, the
interim incident of 8). **Slice 1b built** (br-e949): the wake conditions and `wait-for-wake`
(5), `waiter_grace`. **Slice 2 built** (br-65b8): context and uptime thresholds, the deadline and
the forced restart (6), `bridle handover done`, the `note_tokens`, `plan_tokens`,
`handover_tokens`, `handover_deadline` and `max_uptime` keys. **Slice 3 built** (br-4573): the
handover note as a record (7). The orchestrator
stays an interactive `claude` in the human's tmux pane (the human types to it, locally and over
Remote Control). The daemon keeps it running, tells it when its context or uptime says to hand
over, and carries its wake conditions, with no agent in the loop.

## Rules the human set (fx7x, 2026-09-29)

1. **The pane is the human's.** Bridle types into it only to relaunch
   `bridle session orchestrator --project <project>` when no `claude` is running there. Never into a live session.
2. **Wakes stay in the session:** one background command the orchestrator keeps running, waiting
   on the daemon. If none is waiting, the daemon records an incident.
3. **Forced restart = ask for a handover, then stop the process at the deadline and relaunch.**
   No `/exit` by send-keys.
4. **Crash loops are prevented:** a few relaunches with growing waits, then give up and record an
   incident.
5. **The pane is found by a tmux tag**, never by `session:window.pane` or title.

## 1. Pieces

```
bridle session orchestrator   launches the session; writes the pid file; adds a SessionStart hook (in one --settings object with the lean keys, ct8m)
$BRIDLE_HOME/orchestrator.pid       "<pid> <process start time> <launch epoch>"   (launcher)
$BRIDLE_HOME/orchestrator.session   the current session id                        (the hook)
$BRIDLE_HOME/context/<session id>   context tokens                                (bridle statusline, exists)
daemon: orchestrator supervisor     one task, ticks every 10 s
bridle wait-for-wake                the in-session command
```

The launcher (`bridle session orchestrator`) writes its own pid to the pid file and runs
`claude` as its child, so the pid lives exactly as long as the session. It doesn't `exec`, so that
when `claude` ends it can append the time and exit status (a signal, when above 128) to
`$BRIDLE_HOME/orchestrator.exits` and print it in the pane: sessions had ended unattended with
no trace (fx7x). It passes
`--settings '{"hooks":{"SessionStart":[...]}}'` so that a hook, `bridle orchestrator note-session`
(reads the hook's stdin, writes `orchestrator.session`), runs for this session only (#10). The
hook exists because **`/clear` gives the same process a new session id** (#1, Surprises 1), and
the statusline files are named by session id. The launcher no longer pins `--session-id` or
writes `~/.bridle-orchestrator-session`. The `Stop` hook isn't needed: the design never asks
whether the session is idle (see 5).

The pid file's start time is the check `containment` already uses for agents (pid and start time
must match), so a reused pid is never signalled.

## 2. Config

An `[orchestrator]` table in the project's `.bridle/config.toml` (`roles-and-config.md`, "Other keys").
Absent or `enabled = false`: the supervisor doesn't run (today's behaviour). There is **no pane
key**.

```toml
[orchestrator]
enabled            = true
# launcher        = "..."   # unset: `bridle session orchestrator --project <project>`; else typed verbatim
note_tokens        = "150k"    # "context at N"
plan_tokens        = "180k"    # "plan a handover at the next quiet point"
handover_tokens    = "200k"    # "hand over now"; starts the deadline
handover_deadline  = "30m"     # after the "now" (or uptime) message; then the session is stopped
max_uptime         = "12h"     # a plan-a-handover at this age; the deadline follows
relaunch_backoff   = ["30s", "2m", "10m"]   # wait before relaunch 1, 2, 3; then give up
stable_after       = "10m"     # a session that lasts this long resets the relaunch count
waiter_grace       = "15m"     # no wait-for-wake connected for this long (session up) = incident
```

The pid and session files live under `$BRIDLE_HOME` (default `~/.bridle`), the place the context
files already are; no key. The token thresholds assume the 1M window (`context_window_size`
is in every statusline payload; a percentage form is a follow-up).

## 3. Liveness and the pane

Every tick, if there is a pid file:

- **Alive** = the pid exists and its start time matches. Uptime = now minus the launch epoch.
- **Dead**: go to relaunch (4).

No pid file (never launched by the launcher): the supervisor does nothing and logs once. It
never launches what it hasn't seen run.

**Finding the pane**: `tmux list-panes -a -F '#{pane_id} #{@bridle}'`, the row whose tag is
`orchestrator` (the human ran `tmux set -p @bridle orchestrator` in it). No such pane: record an
incident ("no pane tagged @bridle=orchestrator"), don't guess, and try again next tick, so
tagging the pane fixes it. The pane is looked up only when a relaunch is due, never per tick.

**Safe to type** = the session is dead **and** `#{pane_current_command}` for that pane is a shell
(`zsh`, `bash`, `sh`, `fish`) on two checks 5 s apart (spike #8; the command is the version
string while `claude` runs, so match on shell, not on `claude`). If something else is in the
foreground, don't type; record an incident and retry next tick. tmux missing or not running:
same incident.

## 4. Relaunch and crash-loop prevention

To relaunch: `tmux send-keys -t <pane_id> -l 'bridle session orchestrator --project <project>'` (or `[orchestrator] launcher`, verbatim; the
pane's shell finds `bridle` on its PATH, as the advisor relaunch does), then a separate
`tmux send-keys -t <pane_id> Enter` (spike #7; the two calls are the tested form). The launcher
does `cd`, writes the pid file and starts `claude` with a one-line prompt telling it to run
`bridle prime orchestrator`. Not the prime's text itself: a long prompt in argv matches any
worker's `pkill -f <pattern>`, which is how two sessions died (fx7x). A fresh session, not `--resume`: a handover is how state crosses a restart.

A launch is *confirmed* when the pid file changes to a live pid within 60 s. A launch that isn't
confirmed (the trust prompt of spike Surprise 2, a broken launcher) counts as a failed relaunch
and the pane is then not typed into again until the backoff has passed.

Crash loop: the supervisor keeps `attempts` and `last_attempt` in memory:

- Relaunch *n* (1-based) waits `relaunch_backoff[n-1]` after the previous attempt (the first
  goes at once when the session is found dead, then 30 s, 2 m, 10 m).
- A session that stays up `stable_after` sets `attempts = 0`.
- After `len(relaunch_backoff)` failed relaunches: record an incident ("orchestrator not staying
  up: N relaunches, last attempt <time>") and **stop trying** until a live pid appears (the human
  launching it by hand does that). The daemon restarting resets the count, which is acceptable.
- Built as: the first relaunch goes at once, relaunch *n* after that waits `relaunch_backoff[n-2]`,
  so `len(relaunch_backoff) + 1` relaunches in all before giving up (the incident says how many).
  The two shell checks run on successive 10 s ticks (the first sets a timestamp; a later tick at
  least 5 s on types), so nothing sleeps.
- A relaunch the daemon caused on purpose (section 6) isn't a crash and doesn't count.

When it finds the session dead the supervisor records, in the same incident text or a note to
the human, the exit time as `now` and the transcript's last activity time (`transcript_path`'s
mtime; the path is in every hook payload, so the hook writes it too: `orchestrator.session` holds
`<session id> <transcript path>`). Nothing more: reading the tail for a cause is a follow-up.

## 5. Wake conditions and `bridle wait-for-wake`

The conditions in `scripts/orchestrator-watch.sh` move into the daemon:

| Condition (today's script) | In the daemon |
|---|---|
| an agent exited unexpectedly, crashed or stalled (events) | the event log, from a per-orchestrator cursor |
| a question to the human not yet reported | messages, from a cursor of the last reported message |
| a message to the orchestrator | its inbox: any message with seq past the cursor (no dependence on `--mark-read`) |
| `five_hour` >= 93% or `seven_day` >= 85% | the usage the governor already polls |
| a budget hold begins | the governor's state change |
| a failed CI run on main | the `[ci]` watcher's records (no `gh` polling from the session) |
| context past `CONTEXT_WAKE` | new: 6 below |

Each condition fires **once**, keyed by what it saw (the event seq, message id, CI run id, hold
state), and the cursors are the daemon's own, so nothing is lost or repeated by the session
restarting its watcher. A condition that fires while nobody is waiting waits: **wakes are queued,
not dropped**, so the next `wait-for-wake` returns at once.

**The command.** `bridle agent wake external:orchestrator` is the one wake mechanism; `bridle wait-for-wake` is a thin alias for it (jttf decision 2: same wait, the same reasons and output as before, `--timeout` kept; the mail-only `--mail` went with br-843g). Both are `GET /v1/wake` (the older `GET /v1/orchestrator/wake` still answers the same queue), a long poll (like the
event stream, `api.md`) that the daemon holds until a wake is pending, then answers with the
reasons and their details as JSON, marks them delivered and closes. The command prints them and
exits 0; the orchestrator's Claude Code background task exits, the harness reports the exit to
the model, the model acts and starts the command again, exactly as the script does today. If
nothing is pending the poll is answered empty after 25 min and the command exits 0 with
`nothing` (so a hung TCP connection can't linger); the model restarts it (first thing on any wake, so the gap is seconds). Auth: the
`external:orchestrator` token, as for the other CLI calls; other principals get 403.

**"Nobody is waiting" = an incident.** The daemon counts a waiter as present while a
`wake` request is open, plus `waiter_grace` after the last one closed (so the gap while the model
restarts the command isn't an incident). If the session is alive and there's been no waiter for
longer than `waiter_grace`, record an incident once ("orchestrator has no wake command running");
close it when a request arrives. The session isn't restarted for this: it is still the human's
conversation, and the incident tells the human.

**Built as** (`wake.rs`): the conditions that are facts in the event log (exits, crashes and stalls,
`message.sent` to the orchestrator or a question to the human, a failed `ci.completed`, a
`budget.state` away from `normal`) are derived from events after the cursor, which is one row in
the `meta` table (`orchestrator_wake_cursor`, no schema change) and moves only when wakes are
delivered, so a daemon restart re-derives what was queued. Idle and usage are states
kept in memory (a restart resets their baselines). The wake loop runs whether or not `[orchestrator]`
is enabled. The waiter incident is measured from the later of the last request's close and the
session's launch. `bridle status` shows `waiter_open` and `last_wake_at` (when a poll last
answered with wakes; in memory, so a daemon restart clears it).

**Wakes are also mail (3haz slice 5, br-rhba).** When a daemon raises a wake for the
orchestrator and holds a visitor token `external:orchestrator@<machine>` with a home (the
orchestrator lives on another daemon), it also queues one `system` message per wake and home in
its outbox, addressed to `external:orchestrator`, body `[<reason>] <text> (as of <time>)`, from
`system@<machine>`. The home daemon's `/v1/forward` delivers it like any message (once: the
origin id is deduplicated), so its ordinary message wake wakes the orchestrator's single waiter
there; one waiter on the home daemon replaces one per project. Not forwarded: `message` and
`question` (already messages) and `daemon_stopping`. Each message states when it was true, since
usage, holds and stalls can be over by the time it is read. The local wake path is unchanged
and still answers a waiter on the project's own daemon, so existing per-project waiters keep
working; a daemon that is the orchestrator's home has no visitor token and sends nothing. A
project daemon being down cannot report itself: that is the home outbox's unreachable-peer
alert. The `system` kind is additive (older builds read it as `note`) and is never counted as
the human's unread.

The old shell scripts are deleted, and with them the
`~/.bridle-orchestrator-{seen-questions,hold-state,ci-seen,ctx-level,session}` files. The
role file's watcher step (`workflow/base/roles/orchestrator.md`) becomes: run `bridle
wait-for-wake` in the background, and on any exit, read what it printed, act, and run it again.
The watcher can stay a subagent, as the human agreed; what it runs is now one command.

## 6. Context, uptime and the restart

Context tokens come from `$BRIDLE_HOME/context/<current session id>` (the statusline, spike
#5), the id from `orchestrator.session`. The fallback is the transcript's last `assistant`
entry's `message.usage` (input + cache creation + cache read, #6), read only when the context
file is missing or older than the transcript. The last reading is per session id; a **lower**
reading than before (`/compact`, `/clear`) resets the notes.

| At | The daemon delivers (as a wake) | Then |
|---|---|---|
| >= `note_tokens` (150K) | `context at 150K of the window` | nothing |
| >= `plan_tokens` (180K) | `plan a handover at the next quiet point` | nothing |
| >= `handover_tokens` (200K) | `hand over now; the session is stopped at <deadline>` | deadline starts |
| uptime >= `max_uptime` | `uptime N h: plan a handover at the next quiet point; stopped at <deadline>` | deadline starts |

Each once per session id and crossing. The notes are wakes, so they reach the model through the
same in-session command as everything else; a model that isn't waiting gets the incident of 5,
not a keystroke. **Context tracking** (ct8m step 6): in parallel, the supervisor emits
`orchestrator.context` events (session id, tokens, window size, uptime) on the first reading,
on a lower reading (compact), and at most once per 10 minutes when the tokens change, for
querying with `bridle events --kind orchestrator.context` to answer "how long can the orchestrator
run" with actual data.

**Interactive sessions (advisors, jttf).** `bridle session advisor [name]` registers with the
daemon at launch (`POST /v1/sessions`: identity `advisor` or `advisor/<name>`, the launcher's pid
and start time, the tmux pane) and ends at exit (`POST /v1/sessions/end`), best effort with a 3 s
wait: a daemon that is down never blocks or fails the session. The advisor's SessionStart hook
(`bridle session note`) adds the Claude session id, which changes on `/clear`. The registry is
kept in the daemon's state directory, `sessions.json` beside its database (rewritten on every change, read at daemon start, so a
self-upgrade restart forgets no session; a stopgap until the seats table, gtzx). Every tick (10 s) the daemon drops sessions whose pid is gone or whose start time differs, a reused pid (`session.ended`),
reads each live session's `$BRIDLE_HOME/context/<id>` file and emits `session.context`
(`identity`, `session`, `tokens`, `threshold`, `step`) once per step reached (below), re-armed by
a lower reading. Tokens per session show in `bridle status`
with project, machine, uptime and last activity (the context file's mtime) and in
`GET /v1/sessions`. `bridle session aide` registers the same way (identity `aide`). No
wakes for advisors yet.

**Context steps for every interactive session (jttf).** Not the orchestrator's thresholds: the
human's, `[sessions] warn = ["150k", "200k", "250k", "300k"]` (four increasing counts; per role
in `[sessions.advisor]` and `[sessions.aide]`). Reaching a step sends a `system` note to the
session and one to the human, through `external:aide`, and asks again at every step; a reading
that jumps steps announces only the highest.

| Step | Context | Session is told | Human is told |
|---|---|---|---|
| 0 | 150k | its size | the size, and the restart commands |
| 1 | 200k | restart yourself: say "I'm at 200k; restarting", run `bridle session restart <identity>`, write the handover note it asks for (`bridle handover write --file -`); the human may say keep going | `bridle session keep` to carry on, `restart --fresh` for no handover |
| 2 | 250k | the normal ceiling, past due: restart yourself as at 200k | the same |
| 3 | 300k | the hard limit: write the note now | it is being restarted |

`bridle session keep <identifier>` (`POST /v1/sessions/keep`) is the override: recorded as
`session.override`, told to the session, and the next step says so and asks again. It is refused
before the first step and at step 3. The session has the authority to act on step 1 itself (gq9r), so it no longer waits on the human;
the prompts in `workflow/base/roles/aide.md` and `advisor.md` say the same. Role prompts are read
from the workflow dir when a session launches (`docs/design/workflow-layers.md`), so existing
projects get them at the next launch with no per-project change, unless a project sets its own
`system_prompt`. Nothing is forced below the hard limit. Restarting without a handover stays the human's choice
(`session restart --fresh`) at any point. At step 3 the daemon runs `bridle session restart
<identity>` itself (a handover first, up to the command's 10 minute wait; `--fresh` if the note
never comes) in the repo, and tells aide the outcome, including "run this in a terminal" when
the session has no pane. A registered `aide` session is the human's channel, so its own
warnings are one note. Rejected: percentage thresholds (the window size isn't known per session),
and relaunching from the daemon without the CLI (the pane logic lives in the command).

**Restart on request (jttf).** `bridle session restart <identifier> [--handover|--fresh]`, run by
the human (or the orchestrator for a handover): `--handover`, the default, messages the session to
write a note with `bridle handover write --file -` and waits for a new note of that identity
(10 min, then it fails and nothing is restarted); `--fresh` skips that and is refused inside a session
(`BRIDLE_AS`). Then it SIGTERMs the launcher's children, waits for the launcher to exit and runs
`bridle [--project p] session advisor [name]` in the registered tmux pane with `send-keys`; with
no pane, or a pane that is gone, it prints the command. The new launcher adds the newest note's id
to its opening prompt (`bridle handover show <id>`). The old `~/.bridle/handover/*.md` files are
obsolete and ignored (they were one per identity across all projects, so two projects' aides
collided). Not for the orchestrator (its own handover above); no crash restart (tabled).

**Who talks to the human (r8kv).** The orchestrator session doesn't: it reaches the human only by
messaging `external:aide` (`bridle session aide`, `workflow/base/roles/aide.md`), which
reads the human's to-dos, the workforce's questions and `bridle status`, and relays the human's
answers back, quoted. Advisors only talk, research and file tickets. The daemon's `system` notes
and `question` wakes still go to `human` and `external:orchestrator` as before; moving them to
aide is a later slice.

**Handover done.** The orchestrator writes its state with `bridle handover write` (the one
command for every agent: it records the note, then signals the restart of the caller's own
session; `--no-restart` records only). As the orchestrator that sends
`POST /v1/orchestrator/handover` (the old `bridle handover done` is a deprecated alias for it),
which marks "handover done for this session". At any point,
not only after a message: an orchestrator that hands over early is fine.

**The restart** happens when either the handover is marked done (the session is stopped at
once, there's nothing left to wait for) or the **deadline passes** (30 min after the "now"
or uptime message: it may interrupt a conversation, and that's the bound the human agreed
to; there's no idle detection, so the `Stop` hook isn't used). To stop: `SIGTERM` to the
recorded pid's children, i.e. `claude` (the pid is the launcher script's; TERM to it alone orphans
`claude` on the pane's tty, csfe), or to the pid if it has none; `SIGKILL` to both after 15 s (if
start time matches). The pane is a shell again;
the relaunch of 4 runs at once and isn't counted as a crash. The new session starts from `bridle
prime orchestrator`, which prints the handover.

A forced stop without a handover leaves the previous handover (or the state file) as the
newest: the new session starts from stale state and the incident text says so. That is why the
deadline is long and the two earlier notes exist.

**Built as** (`orchestrator.rs`): the notes are wakes with reason `context`, pushed into the
wake queue (`Wakes::push`). A threshold announces once per session id until a lower reading
resets it; a session id change (`/clear`) starts fresh. Context events are emitted separately
via `emit_context_event()`: on the first reading (when `last_event` is `None`), on a lower
reading (compact), and at most once per 10 minutes when the tokens change, storing the emit
time in `last_event` to throttle further events and detect compacts. `bridle handover done`
sets an in-memory marker with its time; only a marker made after the current process launched
stops it, so a stale one never stops the next session. The stop is the marker or the deadline
(started by the first "hand over now" or uptime note; not restarted by later ones): SIGTERM to
the recorded pid (start time must match), SIGKILL 15 s later if it is still there. Once the
process is gone the ordinary dead-session path relaunches it, flagged deliberate: no backoff,
not counted, and it can't give up; only that first relaunch is free (an unconfirmed one counts
as a crash). A deadline stop says in its incident text that the new session starts from stale
state; a handover stop is not an incident. The transcript fallback reads the last 1 MB of the
file. The deadline is in memory: a daemon restart forgets it (section 8).

## 7. The handover note as a record

Today `docs/context/orchestrator-state.md` is a file the orchestrator commits and pushes, which
fails whenever pushing does. Most of it restates what bridle already holds (`agents`, the queue,
tasks, CI), and that is printed live by `bridle prime orchestrator`. What is left is a note.

- A table `handovers(seq INTEGER PK AUTOINCREMENT, id UNIQUE, role, project, body, created_at,
  created_by)` in the store (`storage.md` gets it; a new schema version). `id` is `h-0007` from
  `seq`, like messages.
- `bridle handover write --file <path>|-` inserts a row for any principal: `role` is the
  writer's own identity from its token (`orchestrator`, `aide`, `advisor/<name>`,
  `agent:<name>`; the human's notes are the orchestrator's), never from the request, plus the
  daemon's project, so each project's daemon keeps its own and each named advisor is its own
  role. `list --role R` and `latest [--role R]` (`?role= on `GET /v1/handovers` and
  `/latest`) read one identity's. **Latest wins, history kept**: prime
  reads the highest `seq`; older rows stay for `bridle handover list` and `show <id>`, pruned
  with the events at 30 days but always keeping each role's newest.
- `bridle handover write` also sends the restart marker (6) for the writer's own session; a note
  written early uses `--no-restart`. Interactive sessions restart through `session restart`'s code;
  workers only record.
- `bridle prime orchestrator` prints the note under a heading, its age, then the live views.
- Also on the state branch, unlike messages and incidents: each note is written as
  `handovers/<id>.md` (TOML frontmatter with id, role, project, created_at, created_by; then the
  body) by the same batched flush, so it is pushed with it (we2r). Files are history, never
  rewritten; the newest by seq is current. `bridle rebuild` restores the table from them, ids
  and seq kept, so new notes number above the highest. Notes already in SQLite are written on
  the daemon's next start if missing. The human's **decisions stay in the repo** (rules,
  tickets, the role file), where they're reviewed.
- Built (slice 3). Prime prints the note's heading and age, then the startup steps, which point
  at the live views (`bridle status`, `agents`, the queue) rather than embedding them. With no
  note, prime prints the state file's pointer (`docs/context/orchestrator-state.md`).

## 8. Errors and small decisions

- The daemon down: the session keeps running; `wait-for-wake` fails, the model sees the error
  and retries. On start the supervisor reads the pid file and carries on; notes already
  delivered aren't repeated because the crossing state is rebuilt from the current reading
  (a session already past 150K gets one "context at N" and a "plan" message on the first tick).
- Cursors and per-crossing state are in memory except the wake cursors (a single row in the
  store, so a daemon restart doesn't replay every old event). The relaunch count is in memory.
- Stopping the orchestrator on purpose (the human quitting it) makes the supervisor relaunch it
  within the first backoff. To stop supervision, set `enabled = false` and restart the daemon.
  A `bridle orchestrator pause` isn't built; it is a small addition if that hurts.
- "Record an incident" here is a `system` message to the human plus an `orchestrator.incident`
  event (`orchestrator.rs`). Incident tasks ([[incidents]], nc7r) are built since, but the
  supervisor doesn't file them; switching it over is not built.
- Not in scope, per the human: the incidents design itself (nc7r), human to-dos (ex9q), and the
  follow-up suggestions in fx7x (launchd, `--resume` after a crash, pid identity beyond start
  time, percentage thresholds).

## 9. Build slices

Each finishes on one branch. Files named are where the change lands.

**Slice 1: liveness, relaunch and the wake command** (replaces `orchestrator-watch.sh`).
- `crates/bridle-daemon/src/orchestrator.rs` (new): supervisor task, pid check, tmux
  wrapper (`list-panes`, `send-keys`, behind a trait so tests fake tmux), relaunch and
  backoff, wake conditions and cursors, waiter tracking.
- `crates/bridle-daemon/src/config.rs`: `[orchestrator]` (`enabled`, `launcher`,
  `relaunch_backoff`, `stable_after`, `waiter_grace`), validation.
- `crates/bridle-daemon/src/server.rs` (route `GET /v1/orchestrator/wake`), `store.rs` (wake
  cursor row, schema version), `main` wiring where the background loops start.
- `crates/bridle-api/src/types.rs` (wake response) and `client/`; `crates/bridle/src/cli.rs`
  (`wait-for-wake`, `orchestrator note-session`).
- `crates/bridle/src/session.rs` (orchestrator launcher, pid file, `--settings` hook);
  `workflow/base/roles/orchestrator.md` (the watcher step).
- Docs: this file, `daemon.md` (loops), `api.md`, `cli.md`, `roles-and-config.md`, `storage.md`.

**Slice 2: context and uptime thresholds, forced restart.** After slice 1; the same files.
- `orchestrator.rs`: context reading (context file, transcript fallback), the notes, uptime,
  the deadline, stop by pid (`SIGTERM`, `SIGKILL`), the deliberate-relaunch flag.
- `config.rs`: `note_tokens`, `plan_tokens`, `handover_tokens`, `handover_deadline`,
  `max_uptime`.
- `server.rs`, `types.rs`, `client/`, `cli.rs`: `bridle handover done` (the marker; no note yet).
- `workflow/base/roles/orchestrator.md` (the context step).
- Docs: `orchestrator-supervision.md`, `cli.md`, `api.md`.

**Slice 3: the handover note as a record.** After slice 2 in the merge order, but independent
of it in code (slice 2's `handover done` is the marker it keeps).
- `store.rs` (table, schema version), `server.rs` (`/v1/handovers`), `types.rs`, `client/`,
  `cli.rs` (`handover write|list|show`), and `crates/bridle/src/prime.rs` printing the note.
- `workflow/base/roles/orchestrator.md`; `docs/context/orchestrator-state.md` shrinks to a
  pointer, then goes.
- Docs: `storage.md`, `cli.md`, `api.md`.

**Same files:** slices 1 and 2 both change `orchestrator.rs`, `config.rs`, `server.rs`,
`types.rs`, `client/`, `cli.rs`, `orchestrator.md` (role), so they go one after the other on
one worker, or slice 2 rebases on slice 1. Slice 3 touches `store.rs`, `server.rs`, `types.rs`,
`client/`, `cli.rs` and the role file, which overlap slice 1's (`store.rs` schema version,
`server.rs`, `types.rs`, `cli.rs`) only at the edges (separate route, table and subcommands):
land it after slice 1 to keep the schema version sequential, and it needn't wait for slice 2.
