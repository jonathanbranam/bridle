+++
id = "br-rdag"
title = "Migrations: automatic runs leave their changes uncommitted, opt-in ones are never run, and the branch guard misfires"
kind = "bug"
state = "built"
created_at = "2026-10-18T23:46:34.284Z"
updated_at = "2026-10-19T11:59:23.007323Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wrdag",
]
branch = "bridle/wrdag"
commit = "4189c3eb5215e071e51e248f8379c8954a172ec4"
summary = """Start-up migrations now commit their output (migrate.rs commit_leftovers, run_at_startup; serve.rs files the to-do). After apply, only the reported files plus .bridle/migrations.toml/.log are committed with "git commit -- paths" (message names the ids); with nothing applied the same step catches up toml/log left dirty earlier. Fail safe: only in a repo rooted at the project dir and only where the existing protected-branch guard allows; otherwise nothing is committed and a chore to-do (what is left, the git add/commit line) is filed. Caveats: a to-do repeats each start until committed; non-state files an old run edited (e.g. dalek-home .gitignore) are not guessed at. Item 3: the guard already followed [branches] integration since br-p4f2 (before the 09:24Z run); dalek-home's main run was allowed because the guard treats a declared integration=main as the project's own branch (I could not see dalek-home's config). Documented in migrations.md. Tests: 3 new (only-its-files, catch-up, integration=dev vs main). Commented on br-jqf9."""
ticket = "rdag"
+++

docs/tickets/open/migrations-automatic-runs-leave-their-changes-uncommitted-op-rdag.md

## Thread

### note · external:aide · 2026-10-18T23:46:40.666Z
Readied by aide: a bug, and the human agreed to the approach for 1 and 2 (rule ready-without-asking).

### note · agent:pm-1 · 2026-10-18T23:47:22.339Z
Brief (pm-1). Right-sized: item 2 (opt-in migrations) moved to its own ticket, to run after this one. THIS ticket is items 1 and 3. Model: Sonnet.
1. Commit what an automatic run changes, as the ask says (only the files the migration changed plus .bridle/migrations.toml and .bridle/migrations.log; message names the migration ids; if it cannot commit, file a to-do for the project's manager saying what is left and how to commit; also catch up leftovers at start). Existing dirty projects (track-web, dalek-home, bridle-ui) are fixed by that same start-up catch-up, so say so in the doc (xebc).
3. Branch guard: find what it did on dalek-home (main) and track-web (dev) and make it follow the project's configured [branches] integration branch. Never touch a project's main or dev beyond what the existing-projects rule allows: on a trial project the commits go on the trial branch.
Overlap: br-jqf9 item 2 (migrations leave the tree dirty) is the same defect; this ticket supersedes it. Docs: docs/design/migrations.md, CHANGELOG. Tests: auto-run commits only its files; dirty-leftover catch-up; guard on a project with integration = "dev". Acceptance: just check. Out of scope: item 2, bridle-ui's 9 uncommitted ticket files (cause unconfirmed; note findings on this thread if the worker sees it).

### note · external:aide · 2026-10-18T23:51:56.980Z
Aide: the dev-vs-main trouble the human heard about on track-web turned out to be ticket ready hardcoding main (filed as br-y7kt), not a migration. track-web's aide committed the 0006/0007 leftovers on dev (704fdc9).

### note · agent:wrdag · 2026-10-19T11:59:22.927Z
built: 4189c3eb5215e071e51e248f8379c8954a172ec4 (branch bridle/wrdag)

### note · agent:wrdag · 2026-10-19T11:59:23.007Z
done: start-up migrations commit their files + catch up leftovers, else file a to-do; just check exit 0, 1893 tests passed; 4189c3eb5215e071e51e248f8379c8954a172ec4
