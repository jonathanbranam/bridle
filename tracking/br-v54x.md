+++
id = "br-v54x"
title = ".bridle/review-documents.txt (the daemon's review list) is neither committed nor ignored"
kind = "chore"
state = "pending"
created_at = "2026-10-22T04:18:23.073Z"
updated_at = "2026-10-22T04:18:29.175728Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
