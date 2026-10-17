+++
id = "br-ux3h"
title = "Waiter: the upgrade exit (code 6) says when (UTC), from/to version and expected downtime"
kind = "feature"
state = "built"
created_at = "2026-10-17T01:14:02.608Z"
updated_at = "2026-10-17T06:54:35.110931Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:ux3h",
]
priority = "low"
priority_at = "2026-10-17T01:14:52.014911Z"
branch = "bridle/ux3h"
commit = "b77fab8e"
summary = "The exit-6 text for an upgrade now adds the UTC time (RFC 3339), 'from <commit>' and, once a restart has been timed, 'back in ~N s'. The daemon stores the structured form as the stop detail and returns it as reasons[].detail ({reason:'upgrade',at,from,to,expected_back_secs}), which --json prints. Restart duration is kept in meta (restart.stopped_at / restart.last_secs, set at restart request and at next start), so the first upgrade after this lands has no expected_back_secs. Changes: server.rs (upgrade_stop + unit test), wake.rs, restart.rs, cli.md, api.md, CHANGELOG. No wire type change (detail already existed). Unit test covers text and detail; exit code 6 path unchanged and covered by existing tests."
ticket = "ux3h"
+++

docs/tickets/open/waiter-the-upgrade-exit-code-6-says-when-utc-from-to-version-ux3h.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T01:14:52.014Z
priority: normal -> low

### note · external:advisor/product-manager · 2026-10-17T01:14:52.409Z
The human, 2026-10-16 ~7:30 PM ET, to advisor (product-manager): "Great yeah seems like a small useful enhancement. But low priority can be done tonight". Readied, priority low; fine to build tonight.

### note · agent:pm-1 · 2026-10-17T01:15:27.280Z
Brief (pm-1): the ask is the brief. Model: Sonnet. Files: crates/bridle-daemon/src/server.rs (set_stop_reason and wherever the upgrade stop reason is built), the wake CLI that prints the exit-6 line and the wire type for the stop reason (crates/bridle-api/src/types.rs, change daemon and clients together), docs/design/cli.md, docs/design/agent-host/daemon.md, CHANGELOG. "expected_back_secs" comes from the last recorded restart duration if one exists; omit it when unknown. Acceptance: just check; tests for text and --json forms and exit code 6. Low priority; it follows the queue. Same wake code as br-8kzq: blocked-by br-8kzq.

### note · agent:ux3h · 2026-10-17T06:54:35.070Z
built: b77fab8e (branch bridle/ux3h)

### note · agent:ux3h · 2026-10-17T06:54:35.110Z
done: waiter upgrade exit has UTC time, from/to, expected downtime in text and --json; just check exit 0, 1793 tests passed, main merged (up to date); b77fab8e
