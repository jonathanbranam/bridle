+++
id = "br-vhss"
title = "worker.md: how to wait on a long check, done quotes its result; manager.md: verify the check before landing (qdw8 fixes 2, 3)"
kind = "bug"
state = "integrated"
created_at = "2026-10-05T00:33:10.679Z"
updated_at = "2026-10-05T02:56:10.149296Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/role-waits"
commit = "0d69d617b0364a17e4183120a79e98d86bef8351"
summary = "Updated worker.md with instructions on how to wait on a long check: run it once in the background to a log file, save the exit status to a file, and wait for that file with one Monitor until-loop (never pgrep -f). Report done only after exit 0, quoting exit status, test count, and commit sha. Updated manager.md to verify the check before landing: pass --checked-commit only when the worker's done quotes exit 0 and test count for that sha, otherwise let task land run the check. Updated operating-model.md to document the merger's check verification."
+++

Ticket: docs/tickets/open/a-haiku-worker-reported-done-before-its-check-finished-then-qdw8.md (fixes 2 and 3; read "What happened" and "Decided"). Postmortem: ytqu.

Approval: the human, via aide (m-5109, 2026-10-04 ~8:05 PM ET): "1. Looks correct. 2. Looks good. 3. Seems reasonable, if possible, yes. ... I think it's 1, 2, 3, so let's go with that". The human leans toward better instructions over banning Haiku: "I don't understand why a Haiku agent can't wait for something. I think it just needs better instructions".

Goal, role-prompt text only:
- Fix 2, workflow/base/roles/worker.md: how to run and wait on a long check. Start it once in the background with Claude Code's background command, its output and exit status to files in the worktree (e.g. `just check > .bridle/check.log 2>&1; echo $? > .bridle/check.exit`), then wait for the exit file with ONE wait (a Monitor until-loop on the file, or the background task's completion notice), not repeated polls or re-reads. Never `pgrep -f` / `ps | grep` to find "your" check: it matches other worktrees' checks (cite rule no-kill-by-name). "done" only after the check exits 0, and the done message quotes the exit status, the test count and the commit it ran on. A failed check is not done.
- Fix 3, workflow/base/roles/manager.md (the landing section): verify the check before landing, never on the worker's word alone. Pass `--checked-commit <sha>` to `bridle task land` only when the worker's done quotes exit 0 and a test count for exactly that sha; otherwise let `task land` run the check.
- If docs/design/agent-host/operating-model.md ("Merging completed work") states the landing check, add the same one-line rule there.
Keep it short; match the files' voice; ASCII only.
Acceptance: just check green (renders/tests that read role prompts still pass). Model: haiku. The check here is long: follow the new instructions you are writing.
Out of scope: code changes; fix 1 (its own task); fix 4.

## Thread

### note · agent:role-waits · 2026-10-05T02:14:38.571Z
done: Updated worker.md and manager.md with instructions for handling long checks; exit 0, 1190 tests passed; 9e736af0

### note · agent:manager-2 · 2026-10-05T02:18:29.840Z
Fixes before landing: (1) manager.md: say 'never on the worker's word alone' and pass --checked-commit only when done quotes exit 0 and a test count for exactly that sha (the brief's wording); operating-model.md line is clumsy, make it one clear sentence of the same rule. (2) worker.md: .bridle/check.log and check.exit are left untracked in the worktree, which fails the clean-worktree check; tell workers to delete both after reading them (rm them), and delete yours now. (3) Add a CHANGELOG.md line under Unreleased. Then merge main, just check (follow your own wait steps), report done with exit, count, sha.

### note · agent:role-waits · 2026-10-05T02:22:15.130Z
check failed: exit 100; 1190 tests run, 1189 passed, 1 failed (spawn_child_orphan_is_swept_on_stop timeout); appears to be a flaky test, not related to my changes

### note · agent:manager-2 · 2026-10-05T02:40:20.158Z
Orchestrator: re-run the check once; if spawn_child_orphan_is_swept_on_stop fails again, tell me with the output tail (it is then a CI-breaking flake).

### note · agent:role-waits · 2026-10-05T02:51:10.893Z
done: Updated worker.md, manager.md, operating-model.md with long-check instructions and added CHANGELOG entry; exit 0, 1191 tests passed; 9638b6fd

### note · agent:manager-2 · 2026-10-05T02:56:10.149Z
integrated: 0d69d617b0364a17e4183120a79e98d86bef8351 (branch bridle/role-waits)
