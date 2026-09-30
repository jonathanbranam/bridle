# Orchestrator supervision

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

1. **The pane is the human's.** Bridle types into it only to relaunch `scripts/claude-orchestrator`
   when no `claude` is running there. Never into a live session.
2. **Wakes stay in the session:** one background command the orchestrator keeps running, waiting
   on the daemon. If none is waiting, the daemon records an incident.
3. **Forced restart = ask for a handover, then stop the process at the deadline and relaunch.**
   No `/exit` by send-keys.
4. **Crash loops are prevented:** a few relaunches with growing waits, then give up and record an
   incident.
5. **The pane is found by a tmux tag**, never by `session:window.pane` or title.

## 1. Pieces

```
scripts/claude-orchestrator   launches the session; writes the pid file; adds a SessionStart hook (in one --settings object with the lean keys, ct8m)
$BRIDLE_HOME/orchestrator.pid       "<pid> <process start time> <launch epoch>"   (launcher)
$BRIDLE_HOME/orchestrator.session   the current session id                        (the hook)
$BRIDLE_HOME/context/<session id>   context tokens                                (bridle statusline, exists)
daemon: orchestrator supervisor     one task, ticks every 10 s
bridle wait-for-wake                the in-session command
```

The launcher (`scripts/claude-orchestrator`, changed) writes its own pid to the pid file and runs
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

A new `[orchestrator]` table in the machine config (`roles-and-config.md`), beside `[budget]`.
Absent or `enabled = false`: the supervisor doesn't run (today's behaviour). There is **no pane
key**.

```toml
[orchestrator]
enabled            = true
launcher           = "scripts/claude-orchestrator"   # relative to the repo, or absolute
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

To relaunch: `tmux send-keys -t <pane_id> -l '<launcher, absolute>'`, then a separate
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
| all agents idle for 15 min | the agent table |
| `five_hour` >= 93% or `seven_day` >= 85% | the usage the governor already polls |
| a budget hold begins | the governor's state change |
| a failed CI run on main | the `[ci]` watcher's records (no `gh` polling from the session) |
| context past `CONTEXT_WAKE` | new: 6 below |

Each condition fires **once**, keyed by what it saw (the event seq, message id, CI run id, hold
state), and the cursors are the daemon's own, so nothing is lost or repeated by the session
restarting its watcher. A condition that fires while nobody is waiting waits: **wakes are queued,
not dropped**, so the next `wait-for-wake` returns at once.

**The command.** `bridle wait-for-wake` is `GET /v1/orchestrator/wake`, a long poll (like the
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
answered with wakes; in memory, so a daemon restart clears it). `scripts/orchestrator-watch.sh` is deleted; `scripts/context-check.sh` was
deleted in slice 2 (with `~/.bridle-orchestrator-{ctx-level,session}`).

`scripts/orchestrator-watch.sh` and `scripts/context-check.sh` are deleted, and with them the
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

**Handover done.** The orchestrator writes its state and runs `bridle handover done`
(`POST /v1/orchestrator/handover`), which marks "handover done for this session". At any point,
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
- `bridle handover write --file <path>|-` inserts a row (role `orchestrator`, the caller's
  project; only the human and `external:orchestrator`). **Latest wins, history kept**: prime
  reads the highest `seq`; older rows stay for `bridle handover list` and `show <id>`, pruned
  with the events at 30 days but always keeping the newest.
- `bridle handover done` (6) is the separate marker, so writing a note early doesn't restart.
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
- Incidents (nc7r) aren't built. Until they are, "record an incident" is a `system` message to
  the human plus an `orchestrator.incident` event, through one function that nc7r replaces.
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
- `scripts/claude-orchestrator` (pid file, `--settings` hook; drop `--session-id`); delete
  `scripts/orchestrator-watch.sh`; `workflow/base/roles/orchestrator.md` (the watcher step).
- Docs: this file, `daemon.md` (loops), `api.md`, `cli.md`, `roles-and-config.md`, `storage.md`.

**Slice 2: context and uptime thresholds, forced restart.** After slice 1; the same files.
- `orchestrator.rs`: context reading (context file, transcript fallback), the notes, uptime,
  the deadline, stop by pid (`SIGTERM`, `SIGKILL`), the deliberate-relaunch flag.
- `config.rs`: `note_tokens`, `plan_tokens`, `handover_tokens`, `handover_deadline`,
  `max_uptime`.
- `server.rs`, `types.rs`, `client/`, `cli.rs`: `bridle handover done` (the marker; no note yet).
- Delete `scripts/context-check.sh`; the role file's context step.
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
