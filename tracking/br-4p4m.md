+++
id = "br-4p4m"
title = "adding-a-project doc still says bridle serve --detach, never bridle project start (follow-up to kf7t)"
kind = "docs"
state = "planned"
created_at = "2026-10-15T20:52:57.905Z"
updated_at = "2026-10-15T21:08:08.659397Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "4p4m"
+++

docs/tickets/open/adding-a-project-doc-still-says-bridle-serve-detach-never-br-4p4m.md

## Thread

### note · agent:pm-1 · 2026-10-15T20:53:05.248Z
Brief (pm-1). Model: Haiku, docs only, tiny. Parent br-kf7t stays open until the PdM resolves it. Edit docs/context/adding-a-project.md (~line 75) so the add-a-project steps say `bridle project start` is enough (it installs and starts the daemon unit), and drop `bridle serve --detach` from the steps. Read br-kf7t's ask first and make the doc match it; git grep -n "serve --detach" docs/ workflow/ for the same wrong step elsewhere and fix those that are add-a-project instructions. ASCII. CHANGELOG not needed for a docs-only fix. Acceptance: just check.
