+++
id = "br-3vv3"
title = "K5: after a stop, the old machine can never serve the project (tools-only entry, registry, upgrade tree)"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T16:16:46.072Z"
updated_at = "2026-10-12T18:07:44.273057Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w3vv3",
]
ticket = "3vv3"
+++

docs/tickets/open/k5-after-a-stop-the-old-machine-can-never-serve-the-project-3vv3.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:36.880Z
Brief (pm-1). Model: Sonnet. Size: S. The ask above is the spec. Blocked by br-mzy4 (K3) only because both edit `project stop`; start after it merges. Files: crates/bridle/src (project stop), crates/bridle-daemon tools_only.rs and the registry/upgrade worktree code, docs/design/cli.md, CHANGELOG.md. Idempotent: running twice changes nothing. Out of scope: deleting bridle.db (K9 handles the return trip).
