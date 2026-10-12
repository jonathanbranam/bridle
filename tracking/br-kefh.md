+++
id = "br-kefh"
title = "K4: project drain: hold new work for one project, no timeout, the orchestrator watching"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T16:16:45.980Z"
updated_at = "2026-10-12T17:55:36.885318Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wkefh",
]
branch = "bridle/wkefh"
commit = "6b3b9a602b693b3f232ad98a62fccf7be0166b52"
summary = "Added bridle project drain [--undo] (crates/bridle/src/project_drain.rs): sets a daemon-side hold (POST /v1/project/hold, held in Status; supervisor refuses spawns, claim_task refuses claims; running agents and messages untouched; separate from the restart 'draining' flag), announces once to every running agent and registered session, polls the K3 stable check (stop_pushes=true, as project stop) with no timeout, messages the orchestrator at start and on each change, names a list unchanged 30 min to orchestrator and aide. Hold is in memory; a running drain re-asserts it after a daemon restart (caveat documented). Test: bridle-daemon/tests/project_hold_test.rs; docs in cli.md, api.md, CHANGELOG. Not tested: the drain loop end to end (only the recipient helper is unit-tested)."
ticket = "kefh"
+++

docs/tickets/open/k4-project-drain-hold-new-work-for-one-project-no-timeout-th-kefh.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:35.508Z
Brief (pm-1). Model: Sonnet. Size: M-L (keep to one branch; if the announce step grows, split it off and tell the manager). Blocked by br-mzy4 (K3): the drain waits on its stable-project check. The ask above is the spec. Start from the existing restart/upgrade drain code in bridle-daemon (refused spawns and claims) and reuse the pieces; do not restart the daemon. Likely files: crates/bridle-daemon (supervisor/server claim and spawn gates, hold state), crates/bridle-api/src/types.rs if a wire type is needed (change clients and daemon together), crates/bridle/src (project drain command), docs/design/agent-host/, docs/design/cli.md, CHANGELOG.md. Out of scope: fixing br-ef3u/br-8swr, a drain timeout, stop/start.

### note · agent:wkefh · 2026-10-12T17:54:55.999Z
built: 2d11b019 (branch bridle/wkefh)

### note · agent:wkefh · 2026-10-12T17:54:56.063Z
done: project drain + hold (spawn/claim refused, announce, stable-check wait); just check exit 0, 1615 tests passed, main merged; 2d11b019

### note · agent:manager-2 · 2026-10-12T17:55:02.418Z
integrated: 6b3b9a602b693b3f232ad98a62fccf7be0166b52 (branch bridle/wkefh)

### note · agent:manager-2 · 2026-10-12T17:55:21.557Z
cleanup: removed agent wkefh, branch bridle/wkefh

### note · external:advisor/product-manager · 2026-10-12T17:55:36.885Z
advisor (product-manager): verified on origin/main (6b3b9a60): project_drain.rs messages the orchestrator at the start and on each change, and names a drain unchanged for 30 min as stuck to the orchestrator and to aide; the hold refuses spawns and claims for this project only (project_hold_test.rs). Matches the ask. Resolving.
