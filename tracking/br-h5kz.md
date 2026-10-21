+++
id = "br-h5kz"
title = "A version 2 ticket that bridle readies on prepare tells nobody: the PM never hears it is open, and open_stale sends it back to pending"
kind = "bug"
state = "integrated"
created_at = "2026-10-21T01:28:33.751Z"
updated_at = "2026-10-21T10:36:24.071062Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:h5kz",
]
branch = "bridle/h5kz"
commit = "225898e7eafc573f5956e5179ea74e173cdd2c9e"
summary = "OpenWatch::ready_effects (readied + filed notice) is now the one place the API ready and the link sync's prepare-ready share; auto_readied adds the state-change event. sync_ticket_links returns the tickets it opened. OpenWatch::tick re-announces a prepared v2 ticket (PM else orchestrator, plus an orchestrator note, once per open_stale period, in-memory throttle) instead of unready_stale. 'Prepared' there = the prepared: true frontmatter flag, as the sync uses. Tests in open_watch_test.rs; docs: stages.md, roles-and-config.md."
ticket = "h5kz"
+++

docs/tickets/open/a-version-2-ticket-that-bridle-readies-on-prepare-tells-nobo-h5kz.md

## Thread

### note · external:advisor/product-manager · 2026-10-21T01:28:45.328Z
Readied by advisor (product-manager): a bug with one clear fix, found on Stages MVP step 6 (br-2ae8). It blocks every version 2 ticket from being planned without a hand nudge.

### note · agent:h5kz · 2026-10-21T10:35:31.798Z
built: f010ace8 (branch bridle/h5kz)

### note · agent:h5kz · 2026-10-21T10:35:32.493Z
done: v2 prepare-ready now runs the API's ready effects (PM message, event, filed notice); open_stale re-announces a prepared v2 ticket instead of unreadying; just check exit 0, 1944 tests passed; f010ace8

### note · agent:manager-2 · 2026-10-21T10:35:49.518Z
integrated: 225898e7eafc573f5956e5179ea74e173cdd2c9e (branch bridle/h5kz)

### note · agent:manager-2 · 2026-10-21T10:36:24.071Z
cleanup: removed agent h5kz, origin branch bridle/h5kz, branch bridle/h5kz
