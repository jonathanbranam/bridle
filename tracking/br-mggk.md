+++
id = "br-mggk"
title = "ticket new: --project writes into the caller's clone, a refused mint leaves a file, --from-task re-links an existing row"
kind = "fix"
state = "integrated"
created_at = "2026-10-17T03:56:57.054Z"
updated_at = "2026-10-17T06:20:45.417854Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:mggk",
]
branch = "bridle/mggk"
commit = "75355ad6dd28204964011fd002f109623f53b655"
summary = "ticket new (crates/bridle/src/ticket.rs): --project X from another project's clone is refused (check_project_matches); a refused mint or failed from-task link removes the file (discard_unfiled; the refusal names repos); --from-task refuses a row whose ticket already has a file (check_not_linked), which also covers br-b3ef. Unit tests for each; project_resolution_test's 'names' no longer reads the file since the fake daemon's 500 now leaves none. Docs: cli.md, CHANGELOG. Not done: ticket check naming a row linked to a missing file (optional in b3ef)."
ticket = "mggk"
+++

docs/tickets/open/ticket-new-project-writes-into-the-caller-s-clone-a-refused-mggk.md

## Thread

### note · agent:pm-1 · 2026-10-17T03:57:00.735Z
Brief (pm-1), split from br-j86p step 2 (not done there). Model: Sonnet. Three bugs in `bridle ticket new`; find the code via crates/bridle (ticket new) and docs/design/cli.md.
1. `ticket new --project X` writes the file into the caller's cwd clone, not X's clone. Fix: write into X's clone, or refuse with a clear error.
2. A refused mint (e.g. forbidden from the daemon) leaves an untracked file. Fix: check/mint with the daemon before writing, or remove the file on refusal.
3. `ticket new --from-task <row>` on a row that already has a ticket file re-links the row to the new id; dropping the duplicate orphans the row (br-cah9 was lost this way). Fix: refuse when the row already has a ticket, or keep the existing link.
Tests for each. Docs: cli.md, CHANGELOG. Acceptance: just check. Out of scope: any token or visitor-rights change.

### note · external:advisor/product-manager · 2026-10-17T03:57:18.977Z
Readied by advisor (product-manager), 2026-10-16 ~11:40 PM ET, under the human's approval of br-j86p (step 2 was part of it: "It sounds right yes. Go ahead" / "Yes") and their 10:48 PM ask to push tickets through. Item 3 is the same bug as br-b3ef (manager-2's, pending): when mggk lands, b3ef can be dropped as a duplicate; its thread also asks that 'bridle ticket check' name a row linked to a missing ticket file (optional, only if small).

### note · agent:mggk · 2026-10-17T06:02:16.535Z
built: b2b5868d (branch bridle/mggk)

### note · agent:mggk · 2026-10-17T06:02:16.647Z
done: ticket new --project refusal, file removed on refused mint, --from-task refuses a linked row (covers b3ef); just check exit 0, 1791 tests passed, b2b5868d

### note · agent:manager-2 · 2026-10-17T06:20:26.714Z
integrated: 75355ad6dd28204964011fd002f109623f53b655 (branch bridle/mggk)

### note · agent:manager-2 · 2026-10-17T06:20:45.417Z
cleanup: removed agent mggk, branch bridle/mggk
