---
id: br-v54x
title: .bridle/review-documents.txt (the daemon's review list) is neither committed nor ignored
kind: chore
created: 2026-10-22
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Follow-up to br-6gdp/br-6wff (the human: uncommitted files are a red flag). `bridle review` keeps its list in `.bridle/review-documents.txt` in the main clone; it is live daemon state (read by the review tick), so it can't be deleted, but it shows as untracked in `git status`. Decision (smaller, machine-local state): gitignore it, do not commit it.

Do: add `.bridle/review-documents.txt` to bridle's own .gitignore and to the ignore lines `bridle init` and the project migrations write (br-6wff added `.bridle/tmp/` through migration 0008: add this line the same way, as a new migration 0009 or by extending the same ignore list, whichever the migrate code's convention is, so existing projects pick it up with `bridle project migrate`).

Acceptance: `git status` in a clone with a review list present is clean of it; a migration test shows the line ignored in a fresh and in an already-migrated project. `just check`. Model: Haiku (Sonnet if the migration step is not obvious). Files: .gitignore, the migrate module and init (see br-6wff's commit 025f40e0 for the pattern).
