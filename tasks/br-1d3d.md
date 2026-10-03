+++
id = "br-1d3d"
title = "bridle land lands one squash commit per task, not --no-ff (mz4q)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T12:04:21.107Z"
updated_at = "2026-09-29T12:15:04.023672Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/land-squash"
commit = "8d078f4"
summary = "bridle land now squash-merges (git merge --squash + commit) in the integration worktree: subject '<task id>: <title>', task summary as body, Task:/Branch: trailers; single parent, so is_merged's Branch: trailer check recognises it. LandInput gained title/summary. A failed squash is undone with reset --hard (no MERGE_HEAD). Manager role/skill and design docs (cli, operating-model, roles-and-config) agree; land_test asserts single parent, trailers and is_merged."
+++

Ticket: docs/questions/open/*mz4q.md (read it, and sq4m/tr7k in docs/questions/resolved/). Goal: bridle land <task-id> lands the task as ONE squash commit on the integration branch in sq4m shape: subject is the task id, colon, the title; the task summary as body; Task: and Branch: trailers. Today crates/bridle-daemon/src/integrator.rs merges with --no-ff. Make workflow/base/roles/manager.md and workflow/base/skills/manager/SKILL.md agree with it (the skill says git merge --squash, the role says --no-ff). Keep is_merged Branch: trailer check working so task done --branch and rm --delete-branch still work (look at how the squash landing was recognised in worktree.rs/supervisor.rs). Do not rewrite the two merges already on main (00af029, 8e41ef6). Update docs/design (cli.md land, agent-host docs) where they state the merge shape. Acceptance: just check passes; test that land produces a single-parent commit with the trailers and that the branch is then seen as merged. Model: Sonnet. Out of scope: hx7t (separate task).

## Thread

### note · agent:land-squash · 2026-09-29T12:14:42.097Z
done: bridle land now makes one squash commit (id: title, summary body, Task:/Branch: trailers); role, skill and docs agree; just check passes; 10686a3

### note · agent:manager-2 · 2026-09-29T12:14:48.935Z
integrated: 8d078f4 (branch bridle/land-squash)

### note · agent:manager-2 · 2026-09-29T12:15:04.023Z
cleanup: removed agent land-squash, branch bridle/land-squash
