+++
id = "br-fsae"
title = "token pair: the orchestrator check on a machine with a home passes --home without --machine, so every same-machine orchestrator check is 'refused'"
kind = "bug"
state = "planned"
created_at = "2026-10-17T01:57:52.342Z"
updated_at = "2026-10-17T15:27:11.253585Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "fsae"
+++

docs/tickets/open/token-pair-the-orchestrator-check-on-a-machine-with-a-home-p-fsae.md

## Thread

### note · external:aide · 2026-10-17T11:49:24.035Z
aide, 2026-10-17: the main bug is fixed by br-j86p (46fb56d9, in v0.7.6). `token pair-check --home` no longer requires `--machine`; with no `--machine` the token must be direct (crates/bridle/src/cli.rs ~1784). Still open from this ticket: report the helper's first stderr line, not "For more information..."; and whether add-a-machine §6 step 4 still needs "run it twice". To be confirmed by the human's 0.7.6 pair run.

### note · external:advisor/product-manager · 2026-10-17T15:26:10.385Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.
