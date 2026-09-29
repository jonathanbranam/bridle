+++
id = "br-3ec1"
title = "Resumed worker dies on its first turn after a daemon restart (p4ks)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T05:20:23.878Z"
updated_at = "2026-09-29T05:35:13.947651Z"
branch = "bridle/resume-fix"
commit = "f88cb9a"
summary = "A --resume process that exits abnormally before any turn ends without is_error is treated as a dead session: finish_agent marks the session unstarted and run_event_task (after removing the runtime) resumes once more on a fresh session (Session::New, same worktree/branch/role, continuation note). agent.exited now carries stderr_tail; warn logged. resume is now boxed like renew (future-type cycle). Renew already recorded the new session id, and the never-started case was already handled; fake-claude gained FAKE_CLAUDE_SESSIONS_DIR to fail --resume of unknown sessions. Ticket p4ks resolved. Note: the suite flakes by timeout under high machine load."
+++

Ticket: docs/questions/open/resumed-worker-dies-on-its-first-turn-p4ks.md. After a daemon restart during a budget hold, a worker showed idle with context 0; each `bridle resume` came up idle, then died on its first turn with is_error, subtype error_during_execution, no tokens, agent.exited code 1 reason eof. Suspected: the stored Claude Code session id is the one renewed away from, or one that never started, so --resume fails.

Do: reproduce with the fake claude (crates/bridle-claude/tests/fake-claude.py; extend it to fail --resume for an unknown session id) and find how the daemon picks the session id on resume/reconcile (crates/bridle-daemon/src/supervisor.rs, store.rs, renew path). Fix so a resume with a dead session id falls back to a fresh session in the same worktree/branch/role with a note message (as renew does), and the failure is logged at warn with the claude stderr tail in the agent.exited event. Make sure a renew records the NEW session id. Record findings in the ticket; resolve it per docs/README.md if fixed.

Acceptance: `just check` passes; a test where resume of a dead session falls back. Model: Sonnet. Out of scope: budget-hold behaviour (br-1392 covers renew during a hold).

## Thread

### note · agent:manager-2 · 2026-09-29T05:35:13.947Z
integrated: f88cb9a (branch bridle/resume-fix)
