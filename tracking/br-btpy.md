+++
id = "br-btpy"
title = "Setup gap (b): bridle project start installs the mail bridge for the project"
kind = "feature"
state = "integrated"
created_at = "2026-10-16T02:24:05.969Z"
updated_at = "2026-10-16T04:31:02.773020Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:btpy",
]
branch = "bridle/btpy"
commit = "34cbe8136511740e7855f687f6dcf78a3c604b12"
summary = 'project start already installed the mail bridge, but only after a take-over (moved-in note). Now carry_in runs the mail step on every start (project_start.rs), prints why it skips (mail off, no [mail], AWS missing, project not in [mail] projects) or "already installed", and a mail failure is printed with the `bridle mail install --project P` command instead of failing the start (mail_step). Test for the failure path; docs add-a-machine/adding-a-project and CHANGELOG updated. Caveat: on a fresh machine the mail token may not exist until token pair; the step runs after pair, so it normally does.'
ticket = "btpy"
+++

docs/tickets/open/setup-gap-b-bridle-project-start-installs-the-mail-bridge-fo-btpy.md

## Thread

### note · agent:pm-1 · 2026-10-16T02:24:17.135Z
Brief (pm-1). Model: Sonnet. Size: S/M. In crates/bridle/src/project_start.rs (the `bridle project start` flow), after the daemon is up and tokens are paired, install and start the project's mail bridge when the project is in `[mail] projects` and the machine has [mail] configured: reuse what `bridle mail install` does (call its function, do not copy it; git grep -n "fn .*install" crates/bridle/src/mail*.rs). Idempotent: if the unit exists and runs, report "mail: already installed". Print what it did or why it skipped (no [mail] config, project not enrolled). A failure to install mail must not fail the start; it prints the exact command to run. Tests with the existing project_start test harness. Docs: docs/context/add-a-machine.md (step 5 no longer manual), docs/context/adding-a-project.md, CHANGELOG. Acceptance: just check. Same file as br-eg64/kf7t (landed); br-b97n edits sync code, not this file. Out of scope: mail config itself, mail install changes.

### note · external:advisor/product-manager · 2026-10-16T02:24:57.792Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".

### note · agent:btpy · 2026-10-16T04:12:35.949Z
built: 88f27b30432c6374508d78cf04b6d32af224161f (branch bridle/btpy)

### note · agent:btpy · 2026-10-16T04:12:36.015Z
done: project start installs mail bridge on every start, non-fatal; just check exit 0, 1734 tests; 88f27b30432c6374508d78cf04b6d32af224161f

### note · agent:btpy · 2026-10-16T04:23:39.202Z
merged main (br-b97n conflict in add-a-machine.md resolved: kept my step 5, their renumbering); just check exit 0, 1736 tests; new tip 263a786d

### note · agent:manager-2 · 2026-10-16T04:30:42.580Z
integrated: 34cbe8136511740e7855f687f6dcf78a3c604b12 (branch bridle/btpy)

### note · agent:manager-2 · 2026-10-16T04:31:02.773Z
cleanup: removed agent btpy, branch bridle/btpy
