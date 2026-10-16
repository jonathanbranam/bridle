+++
id = "br-prf4"
title = "Setup gaps (c, d): bridle init prints 'bridle project start' as the next step and its .gitignore covers bridle's generated .claude files"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T02:24:07.877Z"
updated_at = "2026-10-16T04:54:38.968287Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:prf4",
]
branch = "bridle/prf4"
commit = "0efb9f91e9078f92d81e65a04a7c423214c8f2f2"
summary = """bridle init now prints `bridle project start` as its next step (it printed `bridle serve`). The next-steps text moved into `next_steps()` in crates/bridle/src/init.rs so a test can check it. The .gitignore init writes for a new project also ignores `.claude/skills/bridle-*/` and `.claude/.bridle-sync-hooks.json` (GITIGNORE_LINES, now 5 lines; same append-if-missing behaviour, header now "bridle runtime and generated files"). Tests: next step names project start and not serve; the fresh-init test checks both new lines; the re-run test still shows no duplicates. Docs: cli.md init entry and adding-a-project.md; CHANGELOG line under Unreleased. Caveats: an existing project's .gitignore is not rewritten by this change; a re-run of init on an existing project does append the two lines if missing (as it already does for its other lines). `bridle doctor`'s gitignore check still only looks at the three runtime lines (not changed). The "bridle token pair" line in the next list is kept; project start now pairs tokens itself, so that line may be redundant (not changed here)."""
ticket = "prf4"
+++

docs/tickets/open/setup-gaps-c-d-bridle-init-prints-bridle-project-start-as-th-prf4.md

## Thread

### note · agent:pm-1 · 2026-10-16T02:24:19.285Z
Brief (pm-1). Model: Haiku. Size: S. In `bridle init` (git grep -n "bridle serve" crates/bridle/src/init*.rs; and where it writes .gitignore) change the printed next step to `bridle project start`, and add `.claude/skills/bridle-*/` and `.claude/.bridle-sync-hooks.json` to the .gitignore init writes (copy the exact lines from meta-notes' .gitignore pattern if available in the repo docs; else use these two). Only for new projects; init must not rewrite an existing .gitignore beyond what it does today (it appends missing lines? keep that behaviour: add the two lines the same way). Tests: init output names project start; .gitignore contains both lines; running init twice does not duplicate. CHANGELOG. Acceptance: just check.

### note · external:advisor/product-manager · 2026-10-16T02:24:58.092Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".

### note · agent:prf4 · 2026-10-16T04:42:03.058Z
built: e74a483e (branch bridle/prf4)

### note · agent:prf4 · 2026-10-16T04:42:03.421Z
done: bridle init prints 'bridle project start' and its .gitignore covers .claude/skills/bridle-*/ and .claude/.bridle-sync-hooks.json; just check exit 0 (1735 run, 1735 passed, 6 skipped); e74a483e

### note · agent:prf4 · 2026-10-16T04:48:13.314Z
done (re-merged): main moved to 5c8980cc; merged into bridle/prf4 and re-ran just check: exit 0 (1737 run, 1737 passed, 6 skipped). Tip is a43d911a (merge commit, nothing after the check). Note: my 'ticket built' re-mark was refused for workers, so please land a43d911a from the thread.

### note · agent:prf4 · 2026-10-16T04:48:15.976Z
Re-merged main (5c8980cc) into bridle/prf4 at a43d911a; just check exit 0 (1737 run, 1737 passed, 6 skipped). The re-mark 'ticket built' was refused for workers ('worker agents cannot use agent lifecycle endpoints'); the earlier built (e74a483e) stands, and manager-2 should land a43d911a.

### note · agent:manager-2 · 2026-10-16T04:54:38.968Z
integrated: 0efb9f91e9078f92d81e65a04a7c423214c8f2f2 (branch bridle/prf4)
