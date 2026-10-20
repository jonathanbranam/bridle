+++
id = "br-cgw3"
title = "Each project's usage shares the whole account's weekly gain over its own spend (data-contracts: $0.10 = 60 points)"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T23:11:40.022Z"
updated_at = "2026-10-20T06:37:38.654604Z"
created_by = "external:advisor/usage"
watchers = [
    "external:advisor/usage",
    "agent:wcgw3",
]
branch = "bridle/wcgw3"
commit = "721b4da6b5fbc878521281937edebd6765e8fcee"
summary = "bridle usage --by name|session now sums total_cost_usd over every live daemon in the machine registry (read without a token) and rescales share_points and points_per_dollar by this project's fraction; the note says other machines are not included (a65d) and names unreachable daemons. Gateway already shares over its merged total. Choice recorded in docs/design/usage-and-budget.md. Unit test share_over_machine in usage.rs."
ticket = "cgw3"
+++

docs/tickets/open/each-project-s-usage-shares-the-whole-account-s-weekly-gain-cgw3.md

## Thread

### note · external:advisor/usage · 2026-10-19T23:12:00.735Z
From advisor (usage): filed from the week's usage analysis and readied (kind bug, rule ready-without-asking).

### note · agent:wcgw3 · 2026-10-20T06:37:12.269Z
built: d2ed9b23 (branch bridle/wcgw3)

### note · agent:wcgw3 · 2026-10-20T06:37:12.749Z
done: usage shares points over machine-wide cost; check exit 0, 1941 tests; d2ed9b23

### note · agent:manager-2 · 2026-10-20T06:37:20.699Z
integrated: 721b4da6b5fbc878521281937edebd6765e8fcee (branch bridle/wcgw3)

### note · agent:wcgw3 · 2026-10-20T06:37:24.389Z
Not pushed: 'bridle push-branch' is not a subcommand of the installed bridle binary (unrecognized subcommand; 'bridle push' pushes the integration branch, so not used). Branch bridle/wcgw3 at d2ed9b23 is local only.

### note · agent:manager-2 · 2026-10-20T06:37:38.654Z
cleanup: removed agent wcgw3, branch bridle/wcgw3
