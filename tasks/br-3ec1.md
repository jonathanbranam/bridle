+++
id = "br-3ec1"
title = "Resumed worker dies on its first turn after a daemon restart (p4ks)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T05:20:23.878Z"
updated_at = "2026-09-29T05:20:24.929251Z"
+++

Ticket: docs/questions/open/resumed-worker-dies-on-its-first-turn-p4ks.md. After a daemon restart during a budget hold, a worker showed idle with context 0; each `bridle resume` came up idle, then died on its first turn with is_error, subtype error_during_execution, no tokens, agent.exited code 1 reason eof. Suspected: the stored Claude Code session id is the one renewed away from, or one that never started, so --resume fails.

Do: reproduce with the fake claude (crates/bridle-claude/tests/fake-claude.py; extend it to fail --resume for an unknown session id) and find how the daemon picks the session id on resume/reconcile (crates/bridle-daemon/src/supervisor.rs, store.rs, renew path). Fix so a resume with a dead session id falls back to a fresh session in the same worktree/branch/role with a note message (as renew does), and the failure is logged at warn with the claude stderr tail in the agent.exited event. Make sure a renew records the NEW session id. Record findings in the ticket; resolve it per docs/README.md if fixed.

Acceptance: `just check` passes; a test where resume of a dead session falls back. Model: Sonnet. Out of scope: budget-hold behaviour (br-1392 covers renew during a hold).
