+++
id = "br-eg64"
title = "project start after a take-over doesn't re-pair other machines' peer tokens to the moved project: their messages queue as invalid token"
kind = "bug"
state = "claimed"
created_at = "2026-10-15T00:21:49.137Z"
updated_at = "2026-10-15T01:49:24.715425Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:eg64",
]
ticket = "eg64"
+++

docs/tickets/open/project-start-after-a-take-over-doesn-t-re-pair-other-machin-eg64.md

## Thread

### note · agent:pm-1 · 2026-10-15T00:24:22.861Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body (What's wanted, Why with line refs) is the spec. Change: when `project start` runs the token pair code for P (crates/bridle/src/project_start.rs ~450), the peer phase covers every holder machine whose projects send to P, not only machines that have P (`Fleet::peer_entries`, crates/bridle/src/token_pair.rs ~296: `senders` from `self.up[holder]`); the role-token side stays narrowed to P. The final `token check` (from br-su62) covers those peer entries too, so a stale one shows as refused. Tests: after a simulated take-over (fresh receiver db), start re-mints the other machines' peer entries toward P and the check reports them accepted; a machine with no projects sending to P is not touched. Docs: docs/context/move-a-project.md step 5.3 (adjust to what start now does), cli.md (project start), CHANGELOG.md. Same files as br-8usa: run after it. No migration. Acceptance: just check. Out of scope: reworking `--projects` for plain `token pair`.

### note · agent:eg64 · 2026-10-15T01:49:24.715Z
eg64 (worker): code in token_pair.rs (peer senders and check use all_up), test, CHANGELOG, cli.md, move-a-project.md edited, uncommitted. Targeted test and just check still to run (machine load is high); then merge main, commit, report sha to manager-2.
