+++
id = "br-2ebc"
title = "Design + spike: the daemon supervises the orchestrator session (fx7x)"
kind = "research"
state = "planned"
created_at = "2026-09-29T16:57:45.788Z"
updated_at = "2026-09-29T17:07:56.971352Z"
+++

Ticket: docs/questions/open/*fx7x.md (read all of it; the human's direction and the orchestrator's proposed shape are in the last sections). Also: scripts/claude-orchestrator, scripts/orchestrator-watch.sh, scripts/context-check.sh, crates/bridle/src/statusline.rs (writes ~/.bridle/context/<session id>), docs/context/launchd-restart-plan.md, docs/spikes/01-stream-json-findings.md and docs/spikes/05-stop-hook-findings.md (spike style), the existing per-agent supervision in crates/bridle-daemon/src/supervisor.rs and config.rs. Deliverable is a short spike plus design, no production code: 1) SPIKE, verified on the installed Claude Code (claude --version; live only where it is cheap, on Haiku, in a scratch dir; never touch the real orchestrator session): the SessionStart and Stop hook payloads (fields, when they fire in an interactive session, whether Stop fires per turn), the statusline context_window JSON, the transcript jsonl usage fields as a fallback, and whether tmux send-keys of a one-line prompt into an idle interactive claude submits it and how a /exit behaves. Write docs/spikes/NN-orchestrator-supervision-findings.md with what was observed. 2) DESIGN in docs/design/agent-host/ (new file orchestrator-supervision.md): config ([orchestrator] pane, pid file, thresholds, uptime), liveness via pid, idle/busy via hooks, wake conditions replacing orchestrator-watch.sh, relaunch, context notes at 100K/140K/170K and forced restart after a deadline, uptime restart, and the handover note as a bridle record (latest wins, history kept, printed by bridle prime orchestrator). Keep to KISS and YAGNI (workflow/base/rules/). 3) End with a split into build tasks, each finishable on one branch, naming files: slice 1 = pid check, relaunch in the configured tmux pane, idle wake by send-keys, replacing orchestrator-watch.sh; slice 2 = context and uptime thresholds and the forced restart; slice 3 = the handover note as a record. Say which touch the same files. Update the ticket with the decisions and leave the human's open questions (pane name, thresholds, may a forced restart interrupt a conversation) listed as open with a recommended default. Acceptance: just check passes (docs only). Model: Sonnet. Out of scope: implementing any slice; incidents (nc7r) and human to-dos (ex9q) designs, which are held with the human.

## Thread

### question · external:orchestrator · 2026-09-29T17:07:55.194Z
Held for the human's review of the fx7x design (ticket fx7x, 'The orchestrator's answers and a proposed shape'). Open: the tmux pane name; the thresholds (now 150K note / 210K plan a handover / 255K hand over now, then a forced restart); may a forced restart interrupt a conversation (recommended: wait for idle up to a deadline, then go). Answer to release the task.

### note · agent:manager-2 · 2026-09-29T17:07:56.971Z
STOP now: the human is holding this task. Commit nothing, end your turn.
