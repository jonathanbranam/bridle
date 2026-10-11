+++
id = "br-8ay6"
title = "Direct-to-main docs commits are pushed straight after, on the owner's clone"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T18:08:53.193Z"
updated_at = "2026-10-11T09:35:50.082190Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w8ay6",
]
branch = "bridle/w8ay6"
commit = "9a8ac00c8ce13349c3227e6326b0728b80e1219e"
summary = "Chose mechanism (a), the daemon timer, because it needs nothing installed in the human's clone and covers any committer. New crates/bridle-daemon/src/autopush.rs: every 30s a local rev-list (no fetch) against origin/<integration>; strictly ahead means git push origin <integration> (never a force; behind or diverged is left to k6jd's warning). A failure goes through report_failure (extracted from bridle push, shared): push.failed event plus a note to human and orchestrator, once per distinct error while retrying each tick. Gated on [state] push (existing setting, default on; false turns it off) and on the daemon having started, which already means it holds ownership (8z7j). Tests: temp bare origin+clone (pushes when ahead, leaves in-step/behind alone, rejecting hook gives Err). The event/alarm path itself is only covered via the shared function by the existing bridle push test; no end-to-end timer test. operating-model.md and CHANGELOG updated."
ticket = "8ay6"
+++

Ticket: docs/tickets/open/direct-to-main-docs-commits-are-pushed-straight-after-on-the-8ay6.md (read it; also postmortem j7r4, incident br-2y3m). Model: Sonnet. Blocked by br-8umh (uses its push command and push.failed event).

Goal: on the owner's integration clone, every direct commit on the integration branch (tickets, notes, made by the orchestrator, advisors, aides) is pushed straight after, so the remote never lags the clone, for docs too.

Do:
1. Choose the smallest mechanism and say why in the task comment: (a) the daemon pushes on a short timer when the owner's integration branch is ahead of origin (reuse an existing periodic loop; no new per-tick forks beyond a rev-list, see n4w4), or (b) a post-commit hook installed by bridle. Recommendation: (a), because it needs nothing installed in the human's clone and works for any committer.
2. It uses the br-8umh push command/path, so a rejection records push.failed and alarms. Never force-push. If origin is ahead (a non-fast-forward), do not push; that case is k6jd's divergence warning.
3. Only the owner's clone pushes; non-owner clones never do (ticket 8z7j, read it). Respect a project setting to turn it off if one fits the existing config style (default on); do not invent more config.
4. Fix operating-model.md so "the remote never lags the clone" is true and describes this.
Files likely: crates/bridle-daemon/src (a periodic loop, config.rs), docs/design/agent-host/operating-model.md, CHANGELOG.md.
Tests: temp bare origin plus clone, commit directly, assert pushed within the tick; a rejecting hook gives push.failed. Acceptance: just check passes.
Migration: none for project files (daemon behaviour only); existing projects get it on upgrade. Note the new default in CHANGELOG.
Out of scope: task landings' push (8umh), divergence warnings (k6jd), the one-pusher rule itself (8z7j).

## Thread

### note · agent:w8ay6 · 2026-10-11T09:35:37.147Z
done: mechanism (a), daemon timer (autopush.rs, 30s, local rev-list, reuses [state] push as the off switch). just check exit 0, 1505 tests run, 1505 passed (last full 1501); commit afdcbe2f. Not covered: an end-to-end timer test of the push.failed alarm (shared report_failure is covered by push_test).

### note · agent:w8ay6 · 2026-10-11T09:35:39.151Z
done: daemon autopush of direct commits on the integration branch; just check exit 0, 1505 tests; afdcbe2f

### note · agent:manager-2 · 2026-10-11T09:35:46.476Z
integrated: 9a8ac00c8ce13349c3227e6326b0728b80e1219e (branch bridle/w8ay6)

### note · agent:manager-2 · 2026-10-11T09:35:50.082Z
cleanup: removed nothing; kept agent w8ay6 (background job pid 13784)
