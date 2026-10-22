+++
id = "br-v54x"
title = ".bridle/review-documents.txt (the daemon's review list) is neither committed nor ignored"
kind = "chore"
state = "integrated"
created_at = "2026-10-22T04:18:23.073Z"
updated_at = "2026-10-22T04:57:41.994497Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:v54x",
]
branch = "bridle/v54x"
commit = "1bcce4892d424cf4d5db87014fe47999c1514294"
summary = "Gitignored .bridle/review-documents.txt in .gitignore, init's GITIGNORE_LINES and new migration 0009-gitignore-review-documents (reuses gitignore_line); test covers append-once on an already-migrated .gitignore. No design docs described this."
ticket = "v54x"
+++

docs/tickets/open/bridle-review-documents-txt-the-daemon-s-review-list-is-neit-v54x.md

submitted by external:orchestrator

## The ask

Follow-up to br-6gdp/br-6wff (the human: uncommitted files are a red flag). `bridle review` keeps its list in `.bridle/review-documents.txt` in the main clone; it is live daemon state (read by the review tick), so it can't just be deleted, but it shows as untracked in `git status`. Either gitignore it (it is machine-local state) or commit it on each change (if the list should follow the repo to other machines). Pick the smaller; machine-local + ignore is likely enough.

Acceptance: `git status` in the main clone is clean with a review list present.

Model: Haiku.

## Thread

### note · external:orchestrator · 2026-10-22T04:18:23.074Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-10-22T04:18:54.318Z
orchestrator: readied (chore, rule ready-without-asking). The file was committed by bridle itself (br-gakc working).

### note · agent:v54x · 2026-10-22T04:57:34.541Z
built: 5672129f (branch bridle/v54x)

### note · agent:v54x · 2026-10-22T04:57:34.577Z
done: review list gitignored (bridle, init, migration 0009); just check exit 0, 1967 tests; 5672129f

### note · agent:manager-2 · 2026-10-22T04:57:41.994Z
integrated: 1bcce4892d424cf4d5db87014fe47999c1514294 (branch bridle/v54x)
