+++
id = "br-k6jd"
title = "Managers and the orchestrator fetch origin; divergence from origin is warned (N ahead, M behind)"
kind = "feature"
state = "planned"
created_at = "2026-10-09T18:08:53.045Z"
updated_at = "2026-10-09T22:29:25.128403Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-09T18:09:00.653636Z"
summary = "New divergence.rs: every 10 min (and in bridle daemon doctor) runs git fetch origin in the clone and compares the integration branch with origin/<integration> (rev-list --left-right --count). Any difference (ahead-only included, since 'ahead 23' was the incident) is a git.diverged event {integration, ahead, behind} plus one note to the orchestrator, re-sent only when the counts change; back in step emits zeros and clears. Silent with no origin/branch; doctor warns plainly on fetch failure (daemon logs it). Orchestrator role gets Bash(git fetch origin); managers already had Bash(git *); workers unchanged. Docs: operating-model, api, cli, CHANGELOG (projects overriding orchestrator allowed_tools must add the rule). Migration: built-in role default, so existing projects pick it up on daemon upgrade; no project file hand edit. Check: exit 0, 1418 tests, run on 9b62775b; main merged after (docs-only changes)."
ticket = "k6jd"
+++

Ticket: docs/tickets/open/managers-and-the-orchestrator-fetch-origin-divergence-from-o-k6jd.md (read it; also postmortem j7r4 and incident br-2y3m in docs/context/incidents.md). Model: Sonnet.

Goal: divergence between the local integration branch and origin is visible and reaches the orchestrator, so a clone that is silently far ahead or behind origin cannot hide for hours.

Do:
1. Permissions: managers and the orchestrator may run `git fetch origin` (read-only), including managers in don't-ask mode. Find where the manager/orchestrator allow lists live (workflow/base/roles/manager.md and orchestrator role frontmatter, crates/bridle-daemon/src/config.rs built-in allowed_tools/disallowed_tools, docs/design/agent-host/operating-model.md ~line 160-230 on who may push). Add `Bash(git fetch origin)` (and the same with a remote/branch argument if the existing patterns need it) to those roles only. Workers do not get it. Do not touch `git push` rules.
2. Check: on a timer (reuse an existing periodic loop; do not add a new fork-heavy one, see n4w4) and in `bridle doctor`, the daemon runs `git fetch origin` in the owner's integration clone and compares origin/<integration> with the local integration branch. Warn when origin/<integration> is not an ancestor of local, text "N ahead, M behind" (counts from `git rev-list --left-right --count`). Silent when equal or origin has no such branch or no remote. Fetch failures are reported plainly in the doctor output (do not hide them).
3. The warning reaches the orchestrator as an event (name it `git.diverged`, with ahead/behind counts) and a message, deduplicated: send again only when the counts change, clear when back in step. Follow how other daemon alarms are raised (grep for existing event kinds and the load-hold/state-branch push status code in crates/bridle-daemon/src/state_branch.rs).
Files likely: crates/bridle-daemon/src/{config.rs,doctor-related module,orchestrator.rs}, crates/bridle-api/src/types.rs only if a new event type needs wire support, workflow/base/roles/*.md, docs/design/agent-host/operating-model.md, docs/design/cli.md (doctor), CHANGELOG.md.
Tests: a temp git repo pair (bare origin + clone) in a test; no network. Acceptance: just check passes; doctor shows "N ahead, M behind" in a test.
Migration: role/permission changes come with the workflow base; say in the task comment how existing projects pick them up (bridle workflow sync) and confirm no hand edit of project files is needed. Projects whose .bridle/config.toml override allowed_tools get a CHANGELOG note.
Out of scope: pushing (tasks 8ay6, 8umh), the one-pusher rule (8z7j), worker fetch failure reporting (2mtr).

## Thread

### note · external:advisor/product-manager · 2026-10-09T18:09:00.653Z
priority: normal -> high

### note · agent:k6jd · 2026-10-09T22:20:51.727Z
done: origin divergence watch (git.diverged event + orchestrator note, doctor 'N ahead, M behind'), orchestrator may git fetch origin; just check exit 0, 1418 tests, ran on 9b62775b; main merged after (docs-only), tip 5591a8f8

### note · agent:k6jd · 2026-10-09T22:20:53.978Z
Migration: the permission is a built-in role default in config.rs, so projects get it with the daemon upgrade (no bridle workflow sync or hand edit needed); only projects overriding orchestrator allowed_tools need the CHANGELOG note.

### note · agent:manager-2 · 2026-10-09T22:29:18.702Z
main moved again; merge main into bridle/k6jd (just check only if code changed) and message me the new tip.

### note · agent:k6jd · 2026-10-09T22:29:25.128Z
main merged; only docs/notes/roadmap.md changed, no code, check not re-run. tip 2221134d
