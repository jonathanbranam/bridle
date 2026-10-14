+++
id = "br-85hs"
title = "The 10-minute hold covers all work on a ticket: settle 5m to 10m, and spawns (designer, planner, worker) wait too"
kind = "feature"
state = "planned"
created_at = "2026-10-14T19:32:30.656Z"
updated_at = "2026-10-14T19:33:24.945629Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "85hs"
+++

docs/tickets/open/the-10-minute-hold-covers-all-work-on-a-ticket-settle-5m-to-85hs.md

## Thread

### note · agent:pm-1 · 2026-10-14T19:33:24.478Z
Brief (pm-1). Model: Sonnet. Size: M. The ticket body is the spec. Start: `[tasks] settle` in crates/bridle-daemon/src/tasks.rs ~449 and config.rs (default 5m -> 10m; bridle's own .bridle/config.toml too), and the spawn paths (designer, planner, worker, any agent spawned for a ticket: git grep -n "settle" crates/bridle-daemon/src and the spawn handler in server.rs). Stop points, land in this order and tell the manager if a step gets long: (1) default 10m plus the hold on claim/plan/spawn for a ticket in ANY state incl. pending, refusing with the time the hold ends (refuse, don't queue: simpler); (2) restart the clock on a relayed human word. For (2) look at how the settle clock is stamped today (creation, human comment/edit); a relay comes from an agent comment whose text starts with "From the human" or contains "human via <agent>" (rule human-via-agent): a plain text match is enough, say it is a heuristic in the docs. `skip-settle` unchanged. Careful: critical/urgent paths use skip-settle already; do not break the orchestrator's urgent flow. Tests: spawn for an unsettled pending ticket is held and says when it ends; a settled one spawns; a relayed human comment restarts the clock; an ordinary agent comment does not. Docs: roles-and-config.md ([tasks] settle), the settling section, CHANGELOG.md. Config default change needs no migration (projects without the key get the new default; note it in the CHANGELOG). Acceptance: just check. Out of scope: per-kind hold times.
