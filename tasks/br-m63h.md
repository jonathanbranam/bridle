+++
id = "br-m63h"
title = "A comment on a claimed task doesn't wake the worker that claimed it: the claimant isn't notified like a watcher"
kind = "bug"
state = "planned"
created_at = "2026-10-07T00:48:49.117Z"
updated_at = "2026-10-07T00:49:05.145762Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: m63h
Ticket (the human's relay, what happened, cause, fix, tests; read all of it): docs/tickets/open/a-comment-on-a-claimed-task-doesn-t-wake-the-worker-that-cla-m63h.md
Bug: a plain `bridle task comment` notifies only the task's watchers (a `task_update` message "comment by <actor>: ..."); the claimant is not a watcher, so a manager's send-back comment never reaches the worker doing the task and it sits idle.
EXACT FIX: a comment on a CLAIMED task also notifies its claimant (the `claimed_by` principal), the same message in the same way as for watchers, unless the claimant is the comment's author. Exactly one notification per comment: no duplicate when the claimant is also a watcher, or when the author also passes `--notify <claimant>` (then the --notify message is the one sent, or the watcher one; pick whichever keeps one; say which). Do not change wake policy for idle agents in general; this only adds a recipient. Start at crates/bridle-daemon/src/server.rs ~2867 (the `comment by {actor}` change text and the watcher fan-out) and crates/bridle-daemon/src/principal_wake.rs; tests in crates/bridle-daemon/tests/principal_wake_test.rs show the shape. Also check the same fan-out for `task ask`/`answer` and the other thread notes: if they use the same function the claimant is covered automatically; if not, list them in the done note and do not change them.
Docs: docs/design/cli.md `task comment` line (the claimant is notified too) and docs/design/agent-host/api.md if it lists recipients; CHANGELOG entry (read with a limit).
Tests (test daemon, no real claude): a claimed task; a comment by another principal leaves exactly one unread message for the claimant; the claimant's own comment leaves none; `--notify <claimant>` gives exactly one; a claimant who is also a watcher gets exactly one.
Acceptance: just check passes; the tests above. Migration: none. Model: Sonnet. Out of scope: wake policy for idle agents in general, the manager's send-back flow.
