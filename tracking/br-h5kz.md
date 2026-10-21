+++
id = "br-h5kz"
title = "A version 2 ticket that bridle readies on prepare tells nobody: the PM never hears it is open, and open_stale sends it back to pending"
kind = "bug"
state = "integrated"
created_at = "2026-10-21T01:28:33.751Z"
updated_at = "2026-10-21T12:38:51.521180Z"
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

### note · external:advisor/product-manager · 2026-10-21T10:36:52.373Z
Resolved by advisor (product-manager): 225898e7 on main holds the worker's last done sha f010ace8 (code and tests identical on main). Items 1-4: ready_effects shared by the API and the prepare path (lib.rs auto_readied), open_stale re-announces a prepared v2 ticket, two new open_watch tests, stages.md and roles-and-config.md updated.

### note · external:advisor/tj7q · 2026-10-21T12:38:51.521Z
A second symptom on br-2ae8, after pm-1 planned it (advisor tj7q, 2026-10-21, at the human's request: "yes fix this").

pm-1 planned br-2ae8 (m-17862, open -> planned, ~10:30 PM ET 2026-10-20). Some time before ~11:30 PM ET the row was `built`, with no worker ever claiming it and no implementation branch. `bridle task show br-2ae8` still has `branch bridle/designer-2ae8-proposal` and `commit f9b74cf1` (the proposal stage's integrate from 2026-10-18). manager-2 at 10:06Z on 2026-10-21: "nothing to land ... Nobody has built the feature itself."

So the row of a version 2 ticket carries the stage branch and commit from the design stages, and something takes planned -> built from that (or a stage land marks it built; compare br-fyww, where a stage land marked it integrated). I couldn't see which transition did it; the thread has no note for it.

Please cover it in this fix, or say if it should be its own bug: once a v2 ticket is prepared, its row should start the build clean (no stage branch or commit), and only an implementation worker's hand-off makes it built.
