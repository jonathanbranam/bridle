+++
id = "br-v6cg"
title = "Gateway: account usage summed over every project it reaches (ktgd step 3; takes over br-qhsa's merged route)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:38.482Z"
updated_at = "2026-10-14T17:21:32.395678Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:v6cg",
]
branch = "bridle/v6cg"
commit = "8c9a15abc625a6770e1e989c941e0d295345f819"
summary = "New gateway route GET /api/v1/usage/account?since=&until= (crates/bridle-gateway/src/account_usage.rs, AccountUsage in bindings/). Fans out to every reachable daemon's /v1/usage/groups and /v1/usage/periods; sums role->name->session by key (project/machine null on a row spanning several), re-estimates share_points over the account cost so shares add to seven_day_points; seven_day_points (max across daemons) and periods are taken once; projects[] lists each daemon with reachable/reason, unreachable never zero. br-qhsa left no merged route, so none replaced; per-project usage/history stays. Docs: human-web-ui.md, CHANGELOG; api.md is the daemon API and is unchanged."
ticket = "v6cg"
+++

docs/tickets/open/gateway-account-usage-summed-over-every-project-it-reaches-k-v6cg.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:42.724Z
Brief (pm-1). Model: Sonnet. Size: M. Blocked by br-pemk and br-zn6s. The ask is the spec. Find br-qhsa's merged gateway route first (git log --all --grep=qhsa, or grep crates/bridle-gateway/src for a cross-project usage route) and extend or replace it, keeping its callers working until ui-h79b (bridle-ui) switches. Follow the system.rs/lib.rs patterns (and br-yydm's usage/history route) for fan-out to every project daemon with a fake-daemon test; rate-limit readings come once, not per project; unreachable projects are listed as unreachable. Export TS bindings like the other views. Post the final route path and response shape on this thread as soon as fixed, because ui-h79b waits on it. Docs: human-web-ui.md, api.md, CHANGELOG.md. Out of scope: the UI, other machines' projects the gateway cannot reach.

### note · agent:v6cg · 2026-10-14T09:43:46.319Z
done: gateway GET /api/v1/usage/account, sums projects, unreachable listed, TS bindings; just check exit 0, 1667 tests passed; 57461d52 (main merged after, only a ticket doc moved)

### note · agent:manager-2 · 2026-10-14T09:43:48.673Z
built: 57461d52 (branch bridle/v6cg)

### note · agent:v6cg · 2026-10-14T09:43:49.158Z
Route for ui-h79b: GET /api/v1/usage/account?since=&until= (RFC 3339). Response AccountUsage {projects: ProjectStatus[], since, until, total_cost_usd, seven_day_points, points_per_dollar, share_note, roles: AccountGroup[] (role -> name -> session via children), periods: AccountPeriods|null {timezone, weeks[days], windows, five_hour, seven_day}}. TS in crates/bridle-gateway/bindings/Account*.ts. Always 200; down daemons have reachable=false in projects.

### note · agent:manager-2 · 2026-10-14T17:21:32.395Z
integrated: 8c9a15abc625a6770e1e989c941e0d295345f819 (branch bridle/v6cg)
