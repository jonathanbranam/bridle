---
id: fx7x
title: The orchestrator stays running (a watcher for the watchman)
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-humans-to-do-list-and-restart-checklist-ex9q, where-the-single-orchestrator-lives-hj4g]
tasks: [br-4d5a]
kind: feature
closed: 2026-10-06T00:48:35Z
---

## The ask

The human, verbatim (2026-09-29, via the advisor), on finding the orchestrator gone from
Remote Control:

> Also, I think we need a watcher for the watchman, and we need a cron or system process to be
> sure the orchestrator is running. Having to restart it isn't scalable.

## What happened (advisor, 2026-09-29)

- The orchestrator (`scripts/claude-orchestrator`, session `61113e23-...`, Remote Control name
  `bridle-orch-<hostname>`) was gone: no `claude` process for it at 08:41 ET.
- Its transcript's last real turn is 12:32 UTC (08:32 ET): it sent m-1718/m-1719 to the
  managers (pausing two builds for the incident/open-requests design) and replied to the human.
  The transcript ends at 12:36 UTC with its background task `b71x7i5bz` reported `killed`. No
  error in the transcript and no crash report in `~/Library/Logs/DiagnosticReports`.
- The cause is unknown. Nothing noticed it had gone; the human found out when they went to
  send it a message.
- It's the orchestrator running outside bridle (an `external` principal in a terminal), so the
  daemon's `resume_on_restart` doesn't cover it. The advisor is in the same position.
- **It happened again**: the eleventh orchestrator session (`d19c4df3-...`), idle since its last
  turn at 18:26:54 UTC (2:26 PM), exited at 18:33:18 UTC with no warning and no handover. fx7x
  1a had landed but the daemon wasn't rebuilt, so nothing supervised it. The human restarted it
  by hand.
- **Cause, both times (found 2026-09-29, high confidence): a worker's `pkill -f "just check"`.**
  The launcher passed the whole `bridle prime orchestrator` text to `claude` as an argument; it
  mentions `just check`, and `pkill -f` matches the full command line. Claude Code exits cleanly
  on SIGTERM (hence the "Resume this session with…" banner and no crash report). The
  human-todos worker ran it at 12:36:17.6 UTC and the orchestrator entered its exit handler at
  12:36:17.9 (unified log); orch-1b ran `pkill -P …; sleep 1; pkill -f "just check"` at
  18:33:17.6 and the orchestrator exited at 18:33:18.9. Ruled out: memory (no jetsam), crashes
  (no reports), a Claude Code update (2.1.285 installed at 15:31 ET, after both), the daemon
  (its binary predated fx7x 1a's send-keys), the human (not there; nothing in history.jsonl).
  pkill's target isn't logged, so the match is by timing plus `ps -ww` showing the pattern in
  the orchestrator's argv; the advisor's argv lacked it and it survived.
- **Fixes:** both launchers now pass a one-line prompt; managers told
  every worker never to use `pkill -f`/`killall`, only pids; a worker rule is queued. The
  launcher now logs each exit to `$BRIDLE_HOME/orchestrator.exits`.

## Shape (for the orchestrator to design; KISS)

- Something outside the session (launchd or cron, like the launchd plan for the daemons in
  `docs/context/launchd-restart-plan.md`) checks the orchestrator is running and, if not,
  restarts it, or at least tells the human.
- Find out why it exited this time, if the logs can say.

## The human's direction (2026-09-29, to the orchestrator)

> I think this is a design flaw - you are the lynchpin of too much ... bridle needs to manage
> your session in some way to ensure that you are always safe and running and can then also
> manage context. This should be deterministic part of the system, doesn't need to be agent
> driven. bridle itself can monitor your context and send messages to you about it as well as to
> tell you to write your state out and restart you.

> bridle should monitor you with a script and send messages such as "context filling up,
> currently at x%" and "context reached threshold, please plan a restart" and eventually force a
> restart. Also, it should monitor your uptime as well ... bridle should also I think handle
> this watcher agent that you're using, if possible.

> As for your launch interface, I can assign a specific tmux pane for you and I to interact on
> and bridle should be able to send-keys to that pane to launch the script as needed.

> I would also like to discuss moving your state into bridle instead of a file; it feels like
> something the system should manage.

So: the daemon supervises the orchestrator session the way it supervises its own agents, with
no agent in the loop. The orchestrator stays an interactive `claude` (the human chats with it,
locally and over Remote Control), not a stream-json agent.

## The orchestrator's answers and a proposed shape

**Reading the session without stream-json.** Two sources exist already, both outside the session:
- The status line. Claude Code runs the `statusLine` command on each render with JSON that
  carries `context_window` (used tokens and size); `bridle statusline` already writes that to
  `~/.bridle/context/<session id>` (s8kn), and the watcher reads it (c9zm). It only updates while
  the session renders, which is fine: an idle session's context doesn't grow.
- The transcript, `~/.claude/projects/<dir>/<session id>.jsonl`: every assistant entry has its
  `usage`. It's a fallback that needs nothing from the session. Its mtime is also a
  last-activity time.
- Liveness: the launch script records the `claude` pid next to the session id; the daemon checks
  the pid. Hooks (`SessionStart`, `Stop`) can report started / turn ended, which gives
  idle-vs-busy. Verify the hook payloads against the installed Claude Code before relying on them
  (cite `docs/spikes/01-stream-json-findings.md` style: a short spike).

**The pane is the channel.** The human names a tmux pane (config, e.g. `[orchestrator] pane =
"bridle:0.0"`). The daemon:
- launches `scripts/claude-orchestrator` there when the session is missing (pid gone), and
  records an incident (nc7r) so the human learns of it;
- wakes the orchestrator by typing a one-line prompt there (`send-keys`), only when the session
  is idle (the `Stop` hook's last word), e.g. `bridle: main moved (95adc18); 2 unread`. That
  replaces `scripts/orchestrator-watch.sh` and the background task and cron heartbeat the
  orchestrator runs today: the wake conditions move into the daemon, deterministic, configured
  once. The orchestrator gains nothing from owning its watcher; its background task being
  `killed` is the last thing in the dead session's transcript.
- Risk: typing into the pane while the human is typing there locally. Remote Control messages
  don't collide. Keep wakes short and only on idle; accept the rare collision.

**Context and restarts.** Thresholds (defaults, configurable): a note at 150K ("context at N%"),
at 210K "plan a handover at the next quiet point", at 255K "hand over now", and past a hard
limit (or a handover deadline) the daemon writes the handover marker itself, exits the session
(`/exit` by send-keys, then kill after a grace period) and relaunches. Uptime: the same restart
after N hours even under the context limit, at a quiet point. The handover is done when the
orchestrator runs one command (e.g. `bridle handover done`); the relaunch primes the new session.

**State in bridle, not a file.** Most of `orchestrator-state.md` restates what bridle already
holds: who's running (`bridle agents`), the queue, tasks, CI, and (with ex9q) the human's to-dos.
What's left is a short handover note and the decisions log. Proposal:
- The handover note becomes a bridle record per role and project (latest wins, history kept);
  `bridle prime orchestrator` prints it with live `agents`/`queue`/to-dos. No git commit or
  push per handover (a handover today fails when pushing does).
- The human's decisions stay in the repo (rules, tickets, the role file), where they're reviewed.
- The state file shrinks to a pointer, then goes.

Open for the human: the pane name; the thresholds; whether a forced restart may interrupt a
conversation in progress (proposal: it waits for idle, up to a deadline, then goes anyway).

## Status (2026-09-29)

- Task br-2ebc (spike + design + split) is **held for the human's review**: an open question on
  it blocks it. The first worker was stopped before it began.
- The human raised the thresholds by 50% (from 100K/140K/170K): **150K** note, **210K** plan a
  handover, **255K** hand over now, then the forced restart.
- Still open for the human: the pane name; whether a forced restart may interrupt a conversation
  (recommended: wait for idle up to a deadline, then go).

## The human's review via the advisor (2026-09-29)

The advisor proposed that bridle types into the pane only to relaunch a dead session, never to
wake a live one. The human, verbatim:

> why would bridle every enter text into the pane? Also, i need to talk to someone about build
> progress and system issues, i need to be able to type to the orchestrator.

> yes, definitely add the crash loop prevention. The other suggestions for enhancments can go in
> a suggestions list for follow up later; we're going beying KISS here.

Decided:
- **The pane is the human's.** Bridle types there only to relaunch `scripts/claude-orchestrator`
  when no `claude` is running (a shell prompt, so nobody gets interrupted). It never types into a
  live session.
- **Wakes stay inside the session, with the logic moved to the daemon.** The orchestrator keeps
  one background command (e.g. `bridle wait-for-wake`) that waits on the daemon. The daemon
  decides the wake conditions that `scripts/orchestrator-watch.sh` checks today. If nothing is
  waiting, the daemon records an incident. If the session is gone, it relaunches it.
- **Forced restart without send-keys.** At the hard limit the daemon tells the orchestrator to
  hand over; if it doesn't by the deadline, the daemon stops the process and relaunches it.
- **Crash-loop prevention is in scope.** Allow a few relaunches with growing waits between them,
  then give up and record an incident for the human.

Then, verbatim:

> ok, I see about the orchestator watch issues; so, then let's keep the watcher run as a
> subagent, that is fine; if we can move some of that very complext script into bridle, that
> would be good IMO.

So the in-session watcher stays. What changes is moving the checks in
`scripts/orchestrator-watch.sh` into bridle, so the part left in the session is short.

On the pane name, verbatim:

> I don't care the name, but how do I configure / create this. Can I tag a pane with that name
> right now? Is the pane number tied to layout? That sucks.

Decided (advisor's proposal, answering the human): **tag the pane, not a `session:window.pane`
target.** Pane indexes change when panes are split, closed or moved, and pane titles are
overwritten by programs (Claude Code sets its own). tmux (3.7c here) has per-pane user options:
the human runs `tmux set -p @bridle orchestrator` in the chosen pane. Bridle finds it with
`tmux list-panes -a -F '#{pane_id} #{@bridle}'`. The tag stays with the pane through layout
changes and moves, and is lost only when the pane closes or tmux restarts. If no pane has the
tag, bridle records an incident rather than guessing. No config key for the pane is needed.

The human tagged pane `pi:5.4` (`%39`, running the orchestrator). The advisor checked it and,
at the human's request, typed a note into it. Observed on Claude Code 2.1.284 (one run; the
br-2ebc spike should confirm):
- `tmux send-keys -t %39 -l '<text>'`, then a separate `tmux send-keys -t %39 Enter`, submitted
  the prompt.
- Claude Code's greyed-out suggested prompt was replaced by the typed text, with no mixing.
- `tmux capture-pane -p` drops colours, so it can't tell the suggestion from a draft the human
  typed. A screen check before typing would take a suggestion for a draft. That only matters if
  bridle ever types into a live session, which it won't.

## Suggestions for follow-up (not in scope now; KISS)

- Move the daemons to launchd (`docs/context/launchd-restart-plan.md`, the human's steps) so
  something restarts the daemon too; then check a launchd daemon can reach the tmux server.
- After a crash, `claude --resume <session>` rather than a fresh session; fresh + `bridle prime`
  only after a handover.
- On finding the session dead, record the exit time and the last transcript entries in the
  incident, so the cause isn't lost.
- Check the pid still belongs to `claude` with that session id (pids get reused).
- Thresholds as a percentage of the window, or fail loudly on a small window (the token
  thresholds assume the 1M model).
- The handover note as a bridle record (slice 3) can come later, separately.

## Decisions and design (br-2ebc, 2026-09-29)

Spike: [[docs/spikes/07-orchestrator-supervision-findings|spike 07]]. Design:
[[docs/design/agent-host/orchestrator-supervision|orchestrator supervision]], with the build
slices at its end. The human's answers, all settled:

- **Pane:** found by the tmux tag, `tmux set -p @bridle orchestrator`, read with
  `tmux list-panes -a -F '#{pane_id} #{@bridle}'`. No config key; no tagged pane is an incident.
- **Thresholds:** 150K note, 210K plan a handover, 255K hand over now, then the forced restart.
- **Forced restart:** ask for a handover, then stop the process at the deadline (30 min) and
  relaunch. No `/exit` by send-keys; a restart may interrupt a conversation only after the
  deadline. No idle detection, so the `Stop` hook isn't used.
- **The pane is the human's:** bridle types only to relaunch when no `claude` is running.
- **Wakes** stay in the session: `bridle wait-for-wake`, with the wake conditions of
  `orchestrator-watch.sh` moved into the daemon; an incident if none is waiting.
- **Crash loops:** a few relaunches with growing waits (30 s, 2 m, 10 m), then an incident and stop.

New from the spike: `/clear` gives the same process a new session id, so the session id can't be
pinned at launch; a `SessionStart` hook scoped to the launcher's session records it.

## Resolution

Closed without implementing br-4d5a, by the human's decision (2026-10-05 ~9:20 PM ET, via aide):
"We are closing br-4d5a without implementing it because it has already been built separate. I agree".
Orchestrator supervision slices 1a, 1b, 2 and 3 (br-a424, br-e949, br-65b8, br-4573) built it; the
design as built is [[docs/design/agent-host/orchestrator-supervision|orchestrator supervision]].
The rest (orchestrator pause, percentage thresholds and the like) stays as that design's Planned
items, not this ticket.
