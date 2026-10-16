+++
id = "br-btpy"
title = "Setup gap (b): bridle project start installs the mail bridge for the project"
kind = "feature"
state = "planned"
created_at = "2026-10-16T02:24:05.969Z"
updated_at = "2026-10-16T02:36:09.715354Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "btpy"
+++

docs/tickets/open/setup-gap-b-bridle-project-start-installs-the-mail-bridge-fo-btpy.md

## Thread

### note · agent:pm-1 · 2026-10-16T02:24:17.135Z
Brief (pm-1). Model: Sonnet. Size: S/M. In crates/bridle/src/project_start.rs (the `bridle project start` flow), after the daemon is up and tokens are paired, install and start the project's mail bridge when the project is in `[mail] projects` and the machine has [mail] configured: reuse what `bridle mail install` does (call its function, do not copy it; git grep -n "fn .*install" crates/bridle/src/mail*.rs). Idempotent: if the unit exists and runs, report "mail: already installed". Print what it did or why it skipped (no [mail] config, project not enrolled). A failure to install mail must not fail the start; it prints the exact command to run. Tests with the existing project_start test harness. Docs: docs/context/add-a-machine.md (step 5 no longer manual), docs/context/adding-a-project.md, CHANGELOG. Acceptance: just check. Same file as br-eg64/kf7t (landed); br-b97n edits sync code, not this file. Out of scope: mail config itself, mail install changes.

### note · external:advisor/product-manager · 2026-10-16T02:24:57.792Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".
