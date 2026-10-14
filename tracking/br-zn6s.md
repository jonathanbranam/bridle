+++
id = "br-zn6s"
title = "Daemon usage breakdown for a period: by role, name and session, hosted and external together, with share-of-week estimates (ktgd step 2b)"
kind = "feature"
state = "claimed"
created_at = "2026-10-13T23:05:38.388Z"
updated_at = "2026-10-14T07:00:42.658476Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:zn6s",
]
ticket = "zn6s"
+++

docs/tickets/open/daemon-usage-breakdown-for-a-period-by-role-name-and-session-zn6s.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:40.495Z
Brief (pm-1). Model: Sonnet. Size: M. Blocked by br-pac3 and br-pemk. The ask and ktgd decisions 1 and 4 are the spec. Build on the br-xfy7/br-njjt agent series and list queries (they already union turns and interactive_usage): add a grouped query for a period (cost, tokens, context growth) by role, by name and by session, each row labelled with project and machine, plus the share-of-week estimate (group cost / week cost x the week's seven_day percentage; labelled an estimate in the response) and the measured seven_day points per dollar. One additive route and types (crates/bridle-api/src/types.rs, clients and daemon together). Groups per decision 4, no per-role special cases beyond naming (the human's own sessions as `unbridled`). Test as in the ask (hosted agent with two sessions plus an external session; shares add up to the week's percentage). Docs: api.md, usage-and-budget.md, CHANGELOG.md. Out of scope: gateway (v6cg), CLI (v58q), cross-machine merging (a65d).

### note · agent:zn6s · 2026-10-14T07:00:42.658Z
Built and committed (67546bc0, main already merged). Waiting on just check (running in background); will write summary, mark built and report to manager-2 when it exits 0.
