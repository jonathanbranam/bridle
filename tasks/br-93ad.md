+++
id = "br-93ad"
title = "Push bridle/state to origin after a flush (we2r shape 1), behind a config switch"
kind = "feature"
state = "planned"
created_at = "2026-09-30T00:54:57.873Z"
updated_at = "2026-09-30T00:55:58.482500Z"
size = "S"
+++

CRITICAL (the human). Ticket: docs/questions/open/push-the-state-branch-we2r.md (read it; the Shape and the human's approval question). Read docs/design/storage.md section 'The state branch' and crates/bridle-daemon/src/state_branch.rs (flush_now, the 30 s batching), and how the integrator/land pushes elsewhere for git helpers. Build: after a flush that committed something, push bridle/state to origin (git push origin bridle/state, run against the state branch's own worktree/gitdir as the flush does, never touching the user's checked-out branch), best-effort and debounced (at most one push per N seconds, default 60; one in flight at a time), fast-forward only, NEVER forced; a failed push (network, auth, non-ff) is retried on the next flush or timer, logged at WARN once per change of failure reason, and shown in bridle status (last push time, or 'state push failing: <reason>'), not one incident or message per attempt. A non-fast-forward reject means someone else wrote the branch: stop pushing, show it in status, do not fetch-merge automatically. Config switch in .bridle/config.toml: [state] push = true|false, DEFAULT FALSE (the human has not approved pushing for other projects; workflow/base/rules/existing-projects.md), and set push = true in bridle's OWN .bridle/config.toml only. When push is false, behave exactly as today. Flush must never block on the push (spawn it off the flush path; the runtime must not block: use tokio process). Add remote name config only if needed (default origin). Docs: storage.md ('Pushing on a configurable schedule is not yet built' becomes built, with the switch), daemon/roles-and-config docs for the [state] table, cli.md/status output, CHANGELOG. Acceptance: just check passes; tests against a local bare repo as origin: push after flush, debounce/coalesce, failure shown then retried, non-ff stops pushing, push=false pushes nothing, force is never used. Model: Sonnet. Out of scope: handover notes and rebuild from origin (the next task), pushing for other projects.

## Thread

### note · agent:pm-1 · 2026-09-30T00:55:56.941Z
Cadence from the human (m-2069), overrides the default above: push after every flush that committed something (flushes run at most every 30 s), debounced to AT MOST ONE PUSH PER MINUTE (60 s, not configurable beyond a constant/config default), so origin is never more than about a minute behind: a change made during the debounce window is pushed when the window ends (trailing push), not dropped. Also push once on daemon shutdown, after the final flush, within the shutdown sequence and bounded by a short timeout (say 10 s) so it cannot hang shutdown; failure at shutdown is logged and never blocks exit. Add tests for the trailing push and the shutdown push.

### note · agent:pm-1 · 2026-09-30T00:55:56.957Z
cadence added as a note on the task (push every flush, max one per minute with a trailing push, plus one on shutdown). Read it before starting.

### note · agent:manager-2 · 2026-09-30T00:55:58.482Z
pm-1 added a cadence note on the task: push every flush, max one per minute with a trailing push, plus one on shutdown. Read 'bridle task show br-93ad' thread and follow it.
