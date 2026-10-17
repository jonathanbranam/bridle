+++
id = "br-fsae"
title = "token pair: the orchestrator check on a machine with a home passes --home without --machine, so every same-machine orchestrator check is 'refused'"
kind = "bug"
state = "integrated"
created_at = "2026-10-17T01:57:52.342Z"
updated_at = "2026-10-17T17:20:18.316888Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/wfsae"
commit = "6df627356de4b96497a78d6ae88501e438fbf0c0"
summary = "The same-machine --home refusal was already fixed by br-j86p. Remaining here: a failed pair-check helper is now reported by its first 'error' stderr line (failure_line in token_pair.rs, used for local and ssh runs) instead of clap's 'For more information' trailer; test added to remote_failures_tell_.... Not changed: add-a-machine section 6 step 4 'run twice' (not verified whether still needed; the human's 0.7.6 pair run should confirm)."
ticket = "fsae"
+++

docs/tickets/open/token-pair-the-orchestrator-check-on-a-machine-with-a-home-p-fsae.md

## Thread

### note · external:aide · 2026-10-17T11:49:24.035Z
aide, 2026-10-17: the main bug is fixed by br-j86p (46fb56d9, in v0.7.6). `token pair-check --home` no longer requires `--machine`; with no `--machine` the token must be direct (crates/bridle/src/cli.rs ~1784). Still open from this ticket: report the helper's first stderr line, not "For more information..."; and whether add-a-machine §6 step 4 still needs "run it twice". To be confirmed by the human's 0.7.6 pair run.

### note · external:advisor/product-manager · 2026-10-17T15:26:10.385Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · agent:wfsae · 2026-10-17T17:08:56.578Z
done: helper failures report the error line, not clap's trailer (main bug was already fixed by br-j86p, so no --home change); step 4 'twice' doc left as is, unverified. just check exit 0, 1822 tests passed, 381ddd7b

### note · agent:wfsae · 2026-10-17T17:08:59.819Z
wfsae: committed 381ddd7b; 'bridle ticket built' was refused (forbidden: worker agents cannot use agent lifecycle endpoints), so the manager needs to mark it built.

### note · agent:manager-2 · 2026-10-17T17:20:18.316Z
integrated: 6df627356de4b96497a78d6ae88501e438fbf0c0 (branch bridle/wfsae)
