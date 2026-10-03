+++
id = "br-2ebc"
title = "Design + spike: the daemon supervises the orchestrator session (fx7x)"
kind = "research"
state = "integrated"
created_at = "2026-09-29T16:57:45.788Z"
updated_at = "2026-09-29T17:35:00.465823Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/orch-supervise"
commit = "277e495"
summary = "Docs only. Spike 07 (docs/spikes/07-orchestrator-supervision-findings.md), run on Claude Code 2.1.284 with Haiku in a scratch dir and my own scratch tmux pane (the real orchestrator pane %39 was never typed into): SessionStart/Stop payloads, Stop fires per turn, statusline context_window and the transcript usage fallback (identical numbers), send-keys launches a script in an empty pane, pane_current_command distinguishes a shell from a running claude, and --settings scopes a SessionStart hook to one session. Key surprise: /clear gives the same process a new session id, so the id can't be pinned at launch (the current context-check.sh goes stale); a launcher-scoped SessionStart hook records it. Design: docs/design/agent-host/orchestrator-supervision.md (config, pid liveness, tag-found pane, relaunch with crash-loop backoff, wake conditions moved into the daemon behind `bridle wait-for-wake` with an incident when nothing waits, 150/210/255K notes, uptime, forced restart by SIGTERM at a deadline, handover note as a record) ending in three build slices: 1 pid/relaunch/wake command (new orchestrator.rs, config.rs, server.rs, store.rs, cli.rs, launcher, delete watch script), 2 thresholds/uptime/forced restart (same files as 1, so sequential), 3 handover record (store/server/types/client/cli/prime.rs; land after 1). Ticket fx7x updated with the settled answers. No idle detection, so no Stop hook. Caveat: parses_fast timing test failed once under load, passed alone and on rerun."
+++

Ticket: docs/questions/open/*fx7x.md (read all of it; the human's direction and the orchestrator's proposed shape are in the last sections). Also: scripts/claude-orchestrator, scripts/orchestrator-watch.sh, scripts/context-check.sh, crates/bridle/src/statusline.rs (writes ~/.bridle/context/<session id>), docs/context/launchd-restart-plan.md, docs/spikes/01-stream-json-findings.md and docs/spikes/05-stop-hook-findings.md (spike style), the existing per-agent supervision in crates/bridle-daemon/src/supervisor.rs and config.rs. Deliverable is a short spike plus design, no production code: 1) SPIKE, verified on the installed Claude Code (claude --version; live only where it is cheap, on Haiku, in a scratch dir; never touch the real orchestrator session): the SessionStart and Stop hook payloads (fields, when they fire in an interactive session, whether Stop fires per turn), the statusline context_window JSON, the transcript jsonl usage fields as a fallback, and that tmux send-keys can launch scripts/claude-orchestrator in an empty pane (bridle never types into a live session). Write docs/spikes/NN-orchestrator-supervision-findings.md with what was observed. 2) DESIGN in docs/design/agent-host/ (new file orchestrator-supervision.md): config (pid file, thresholds, uptime; no pane key: the pane is found by its tmux tag), liveness via pid, wake conditions moved into the daemon and delivered to one in-session background command (e.g. bridle wait-for-wake) that the orchestrator keeps running (an incident if none is waiting), relaunch, context notes at 150K/210K/255K (the human raised them 50%) and forced restart after a deadline, uptime restart, and the handover note as a bridle record (latest wins, history kept, printed by bridle prime orchestrator). Keep to KISS and YAGNI (workflow/base/rules/). 3) End with a split into build tasks, each finishable on one branch, naming files: slice 1 = pid check, relaunch in the tagged tmux pane with crash-loop backoff, and the in-session wake command, replacing orchestrator-watch.sh; slice 2 = context and uptime thresholds and the forced restart; slice 3 = the handover note as a record. Say which touch the same files. Update the ticket with the decisions and record the human.s answers (all settled; see below). Acceptance: just check passes (docs only). Model: Sonnet. Out of scope: implementing any slice; incidents (nc7r) and human to-dos (ex9q) designs, which are held with the human.

The human's review (ticket fx7x, 'The human's review via the advisor', c50cd72) overrides the ticket's earlier proposal: (1) bridle types into the pane ONLY to relaunch scripts/claude-orchestrator when no claude is running, never into a live session; (2) wakes stay in-session via a background command waiting on the daemon; the daemon records an incident if nothing is waiting; (3) forced restart = tell the orchestrator to hand over, then stop the process at the deadline and relaunch; no /exit by send-keys; (4) crash-loop prevention is in scope: a few relaunches with growing waits, then give up and record an incident. The ticket's 'Suggestions for follow-up' are out of scope.

Settled by the human (m-1776, fx7x on main): the pane is tagged with a tmux pane user option, `tmux set -p @bridle orchestrator`; bridle finds it with `tmux list-panes -a -F '#{pane_id} #{@bridle}'`, never by session:window.pane index or title. If no pane has the tag, record an incident; don't guess. Thresholds: 150K note, 210K plan a handover, 255K hand over now, then stop at the deadline and relaunch. Crash-loop prevention in scope.

## Thread

### question · external:orchestrator · 2026-09-29T17:07:55.194Z
Held for the human's review of the fx7x design (ticket fx7x, 'The orchestrator's answers and a proposed shape'). Open: the tmux pane name; the thresholds (now 150K note / 210K plan a handover / 255K hand over now, then a forced restart); may a forced restart interrupt a conversation (recommended: wait for idle up to a deadline, then go). Answer to release the task.

### note · agent:manager-2 · 2026-09-29T17:07:56.971Z
STOP now: the human is holding this task. Commit nothing, end your turn.

### note · external:orchestrator · 2026-09-29T17:16:52.172Z
The human (via advisor, m-1774): 'let's keep the watcher run as a subagent, that is fine; if we can move some of that very complex script into bridle, that would be good.' So the session side is one bridle command that waits and prints the wake reason; all the checks live in bridle.

### answer · external:orchestrator · 2026-09-29T17:18:45.487Z
The human answered (via advisor, m-1776): pane found by the tmux tag @bridle=orchestrator (no config key; incident if none); thresholds 150K/210K/255K stand; forced restart = hand over, else stop at the deadline and relaunch; crash-loop prevention in scope. Brief updated. Released.

### note · external:orchestrator · 2026-09-29T17:20:28.878Z
The orchestrator's pane is tagged: tmux pane %39 (pi:5.4) has @bridle=orchestrator, verified from inside the session (TMUX_PANE=%39). The spike must not type into it: it's the live session.

### note · agent:orch-supervise · 2026-09-29T17:33:24.840Z
done: spike 07 (docs/spikes/07-*) + design (docs/design/agent-host/orchestrator-supervision.md) + 3 build slices, fx7x ticket updated; just check green with main merged (parses_fast timing test flaked once under load, passes alone/rerun); key finding: /clear changes session id in same process, so a launcher-scoped SessionStart hook is needed; commit 07669fb

### note · agent:manager-2 · 2026-09-29T17:33:30.747Z
Content fine. main moved: merge main into your branch (no other changes), rerun just check, message me the sha.

### note · agent:orch-supervise · 2026-09-29T17:34:19.311Z
done: main merged again, just check green (684 passed); sha 97ed1bf

### note · agent:manager-2 · 2026-09-29T17:34:23.499Z
integrated: 277e495 (branch bridle/orch-supervise)

### note · agent:manager-2 · 2026-09-29T17:35:00.465Z
cleanup: removed agent orch-supervise, branch bridle/orch-supervise
