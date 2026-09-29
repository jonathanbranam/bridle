+++
id = "br-1d3d"
title = "bridle land lands one squash commit per task, not --no-ff (mz4q)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T12:04:21.107Z"
updated_at = "2026-09-29T12:04:23.630447Z"
+++

Ticket: docs/questions/open/*mz4q.md (read it, and sq4m/tr7k in docs/questions/resolved/). Goal: bridle land <task-id> lands the task as ONE squash commit on the integration branch in sq4m shape: subject is the task id, colon, the title; the task summary as body; Task: and Branch: trailers. Today crates/bridle-daemon/src/integrator.rs merges with --no-ff. Make workflow/base/roles/manager.md and workflow/base/skills/manager/SKILL.md agree with it (the skill says git merge --squash, the role says --no-ff). Keep is_merged Branch: trailer check working so task done --branch and rm --delete-branch still work (look at how the squash landing was recognised in worktree.rs/supervisor.rs). Do not rewrite the two merges already on main (00af029, 8e41ef6). Update docs/design (cli.md land, agent-host docs) where they state the merge shape. Acceptance: just check passes; test that land produces a single-parent commit with the trailers and that the branch is then seen as merged. Model: Sonnet. Out of scope: hx7t (separate task).
