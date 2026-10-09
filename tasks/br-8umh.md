+++
id = "br-8umh"
title = "A failed push is an event and an alarm; an agent that can't send puts the blocker on the task"
kind = "feature"
state = "planned"
created_at = "2026-10-09T18:08:53.344Z"
updated_at = "2026-10-09T18:10:08.954226Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "8umh"
+++

Ticket: docs/tickets/open/a-failed-push-is-an-event-and-an-alarm-an-agent-that-can-t-s-8umh.md (read it; also postmortem j7r4 and incident br-2y3m in docs/context/incidents.md). Model: Sonnet.

Goal: a rejected push after a landing is never dependent on an agent's session text; it is a recorded event and an alarm.

Do:
1. Today the merger runs `git push origin <integration>` itself (docs/design/agent-host/operating-model.md ~line 225, step 4; roles in workflow/base/roles/manager.md). Replace that with a bridle command: `bridle task land <id> --push`, or if landing is not a bridle command today, a small `bridle push` (human, orchestrator and manager only; workers stay denied `git push`). First read how landing/integration is done (crates/bridle/src, crates/bridle-daemon/src/tasks.rs, server.rs) and pick the smaller shape; say which in the task comment.
2. The command runs `git push origin <integration>` in the caller's clone and on failure records a `push.failed` event (project, branch, git's stderr first line) and notifies the orchestrator and the human by message. On success it records `push.ok` only if events for success already exist in that style; otherwise nothing. Exit nonzero on failure so the agent also sees it.
3. Roles: manager and worker role text (workflow/base/roles/): an agent that cannot send a message (`bridle send` fails) puts the blocker on the task thread with `bridle task comment` instead. Update the manager role to use the new push command, and its allow list.
Files likely: crates/bridle/src (CLI), crates/bridle-daemon/src/{tasks.rs,server.rs,config.rs}, crates/bridle-api/src/types.rs (new event kind, wire change: all clients together), workflow/base/roles/{manager,worker}.md, docs/design/cli.md, operating-model.md, CHANGELOG.md.
Tests: temp bare origin whose pre-receive hook rejects, assert exit code, event and message; no network. Acceptance: just check passes.
Migration: roles reach projects via bridle workflow sync; state that in the task comment.
Out of scope: pushing after direct docs commits (task 8ay6, which builds on this command), fetch/divergence warnings (k6jd), incident 2ax5's root cause (why bridle send failed).
