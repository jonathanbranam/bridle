---
id: br-mggk
title: "ticket new: --project writes into the caller's clone, a refused mint leaves a file, --from-task re-links an existing row"
kind: fix
created: 2026-10-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask


Split from br-j86p step 2 (approved by the human with j86p: "It sounds right yes. Go ahead"
and "Yes", 2026-10-16). Three bugs in `bridle ticket new`:

1. `ticket new --project X` writes the file into the caller's cwd clone, not X's clone. Write
   into X's clone, or refuse with a clear error.
2. A refused mint (e.g. forbidden by the daemon) leaves an untracked file. Check/mint with the
   daemon before writing, or remove the file on refusal.
3. `ticket new --from-task <row>` on a row that already has a ticket file re-links the row to
   the new id; dropping the duplicate orphans the row (br-cah9 was lost this way). Refuse when
   the row already has a ticket, or keep the existing link. Same bug as br-b3ef, which is
   dropped as a duplicate when this lands.

Tests for each. Docs: cli.md, CHANGELOG. Acceptance: `just check`. Out of scope: any token or
visitor-rights change.
