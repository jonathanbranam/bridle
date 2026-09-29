+++
id = "br-2ebc"
title = "Design + spike: the daemon supervises the orchestrator session (fx7x)"
kind = "research"
state = "planned"
created_at = "2026-09-29T16:57:45.788Z"
updated_at = "2026-09-29T17:18:45.487096Z"
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
