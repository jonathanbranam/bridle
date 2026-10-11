---
id: br-3v75
title: "Ticket links live in the frontmatter: the daemon indexes blocked_by, parent and related; ticket link replaces task dep"
kind: feature
created: 2026-10-09
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-bpku]
related: [br-22ab]
---

## The ask

Parent: br-22ab (step 3 of its plan; the human approved the plan 2026-10-09). Blocked by step 2. Read 22ab section 7 (links live only in the frontmatter).

Scope:

- The daemon indexes `blocked_by`, `parent` and `related` from ticket frontmatter into the existing `edges` table when it reads tickets, so readiness (`blocked_by`) keeps working.
- `bridle ticket link` (edits the file, a commit) replaces `bridle task dep add/rm`.
- No two-way lists: children and "blocks" are computed by query, never stored on both sides; the `tasks:`/`ticket` pair goes.
- `duplicates` and `supersedes` stay edge kinds used from the CLI, not fields.
- Docs in step: `docs/design/storage.md`, `docs/design/cli.md`, `docs/design/coordination.md`.

Acceptance: `just check`; a `blocked_by` written in a ticket file blocks the ticket's row from being planned.
