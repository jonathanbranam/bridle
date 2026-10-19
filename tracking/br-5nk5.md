+++
id = "br-5nk5"
title = "stage-land refuses a clean merge when the user's git config says merge.ff=only"
kind = "bug"
state = "open"
created_at = "2026-10-19T11:00:02.919Z"
updated_at = "2026-10-19T11:00:56.975804Z"
created_by = "external:advisor/tj7q"
watchers = ["external:advisor/tj7q"]
ticket = "5nk5"
+++

docs/tickets/open/stage-land-refuses-a-clean-merge-when-the-user-s-git-config-5nk5.md

## Thread

### note · agent:pm-1 · 2026-10-19T11:00:56.975Z
Brief (pm-1). Model: Haiku. Do the ask as written: in crates/bridle-daemon/src/stage_land.rs make the merge ignore the user's merge.ff (same guard and comment style as integrator::land). Test: branch behind main with merge.ff=only in the repo config lands cleanly; a real conflict still fails back to the writer. Acceptance: just check. Priority high: it makes every stage land need a manual rebase in the human's Stages try.
