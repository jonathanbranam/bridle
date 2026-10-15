+++
id = "br-4p4m"
title = "adding-a-project doc still says bridle serve --detach, never bridle project start (follow-up to kf7t)"
kind = "docs"
state = "claimed"
created_at = "2026-10-15T20:52:57.905Z"
updated_at = "2026-10-15T21:19:43.769633Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:4p4m",
]
ticket = "4p4m"
+++

docs/tickets/open/adding-a-project-doc-still-says-bridle-serve-detach-never-br-4p4m.md

## Thread

### note · agent:pm-1 · 2026-10-15T20:53:05.248Z
Brief (pm-1). Model: Haiku, docs only, tiny. Parent br-kf7t stays open until the PdM resolves it. Edit docs/context/adding-a-project.md (~line 75) so the add-a-project steps say `bridle project start` is enough (it installs and starts the daemon unit), and drop `bridle serve --detach` from the steps. Read br-kf7t's ask first and make the doc match it; git grep -n "serve --detach" docs/ workflow/ for the same wrong step elsewhere and fix those that are add-a-project instructions. ASCII. CHANGELOG not needed for a docs-only fix. Acceptance: just check.

### note · agent:4p4m · 2026-10-15T21:19:43.769Z
Handoff (4p4m): doc edit committed on bridle/4p4m as 9e6c80da (adding-a-project.md step 3 now says bridle project start; boot-section sentence no longer names serve --detach). Still open: just check was running in the background (log /tmp/br-4p4m-check.log, not yet finished), then the ticket summary, then the sha report to manager-2. Claim kept until the check finishes.
