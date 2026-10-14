+++
id = "br-zn6s"
title = "Daemon usage breakdown for a period: by role, name and session, hosted and external together, with share-of-week estimates (ktgd step 2b)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:38.388Z"
updated_at = "2026-10-14T09:07:31.850198Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:zn6s",
]
branch = "bridle/zn6s"
commit = "b37cc4f2cd71ebe5f31b6242a7bdc95425b44ca7"
summary = "Added GET /v1/usage/groups?since=&until= (types in bridle-api, client method, daemon route, usage_groups.rs, Store::usage_sessions): a period's cost, tokens and context growth grouped role -> name -> session, hosted turns and external reports together, rows labelled project/machine, share_points per group (cost share x the seven_day points the period gained; labelled an estimate) and measured points_per_dollar. Test: hosted agent with two sessions plus an external session; sums at each level and shares add to the week's points. Docs: api.md, usage-and-budget.md, CHANGELOG. Not merged across machines (a65d); gateway and CLI are v6cg and v58q."
ticket = "zn6s"
+++

docs/tickets/open/daemon-usage-breakdown-for-a-period-by-role-name-and-session-zn6s.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:40.495Z
Brief (pm-1). Model: Sonnet. Size: M. Blocked by br-pac3 and br-pemk. The ask and ktgd decisions 1 and 4 are the spec. Build on the br-xfy7/br-njjt agent series and list queries (they already union turns and interactive_usage): add a grouped query for a period (cost, tokens, context growth) by role, by name and by session, each row labelled with project and machine, plus the share-of-week estimate (group cost / week cost x the week's seven_day percentage; labelled an estimate in the response) and the measured seven_day points per dollar. One additive route and types (crates/bridle-api/src/types.rs, clients and daemon together). Groups per decision 4, no per-role special cases beyond naming (the human's own sessions as `unbridled`). Test as in the ask (hosted agent with two sessions plus an external session; shares add up to the week's percentage). Docs: api.md, usage-and-budget.md, CHANGELOG.md. Out of scope: gateway (v6cg), CLI (v58q), cross-machine merging (a65d).

### note · agent:zn6s · 2026-10-14T07:00:42.658Z
Built and committed (67546bc0, main already merged). Waiting on just check (running in background); will write summary, mark built and report to manager-2 when it exits 0.

### note · agent:zn6s · 2026-10-14T07:19:49.026Z
done: GET /v1/usage/groups (role/name/session, share estimates); just check exit 0, 1661 tests passed; 67546bc0

### note · agent:zn6s · 2026-10-14T07:19:51.874Z
bridle ticket built was refused: 'forbidden: worker agents cannot use agent lifecycle endpoints'. Branch bridle/zn6s at 67546bc0 is ready; check exit 0, 1661 tests passed. Manager please mark built/land.

### note · agent:manager-2 · 2026-10-14T07:19:55.052Z
Please write the summary with 'bridle ticket summary br-zn6s' (none on the ticket yet), then message me. I will mark it built.

### note · agent:zn6s · 2026-10-14T07:20:01.687Z
summary written on br-zn6s (bridle ticket summary --file); branch bridle/zn6s at 67546bc0, check exit 0, 1661 tests passed. Ready for you to mark built.

### note · agent:manager-2 · 2026-10-14T07:20:06.773Z
built: 67546bc0 (branch bridle/zn6s)

### note · agent:manager-2 · 2026-10-14T09:07:31.850Z
integrated: b37cc4f2cd71ebe5f31b6242a7bdc95425b44ca7 (branch bridle/zn6s)
