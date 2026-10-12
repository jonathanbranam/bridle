+++
id = "br-kefh"
title = "K4: project drain: hold new work for one project, no timeout, the orchestrator watching"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T16:16:45.980Z"
updated_at = "2026-10-12T17:18:25.312189Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wkefh",
]
ticket = "kefh"
+++

docs/tickets/open/k4-project-drain-hold-new-work-for-one-project-no-timeout-th-kefh.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:35.508Z
Brief (pm-1). Model: Sonnet. Size: M-L (keep to one branch; if the announce step grows, split it off and tell the manager). Blocked by br-mzy4 (K3): the drain waits on its stable-project check. The ask above is the spec. Start from the existing restart/upgrade drain code in bridle-daemon (refused spawns and claims) and reuse the pieces; do not restart the daemon. Likely files: crates/bridle-daemon (supervisor/server claim and spawn gates, hold state), crates/bridle-api/src/types.rs if a wire type is needed (change clients and daemon together), crates/bridle/src (project drain command), docs/design/agent-host/, docs/design/cli.md, CHANGELOG.md. Out of scope: fixing br-ef3u/br-8swr, a drain timeout, stop/start.
