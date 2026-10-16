+++
id = "br-mtdg"
title = "Self-upgrade defaults to release; commit mode only for bridle's own repo"
kind = "bug"
state = "planned"
created_at = "2026-10-16T21:05:17.647Z"
updated_at = "2026-10-16T21:17:03.190719Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "mtdg"
+++

docs/tickets/open/daemon-restart-upgrade-builds-the-project-s-repo-instead-of-mtdg.md

## Thread

### note · agent:pm-1 · 2026-10-16T21:05:55.481Z
Brief (pm-1): the ask above is the brief. Model: Sonnet. Files: crates/bridle-daemon config.rs and the upgrade code (grep self_upgrade), docs/design/agent-host/daemon.md ('Release upgrade'), docs/design/roles-and-config.md, CHANGELOG. Worker decides precedence (suggested: a project's explicit true keeps commit mode over a machine false) and records it in daemon.md. Acceptance: just check; tests for the new default, the precedence, and the one-line refusal on a repo with no crates/bridle.
Migration (no file change): this is a behaviour change on every machine. Before the build is deployed on dalek, the human must set the machine value in ~/.bridle/config.toml, or dalek's non-bridle daemons will start installing releases. The worker puts that step in the CHANGELOG and the ticket's done line; the manager tells the orchestrator before the upgrade. Out of scope: release publishing (br-9a7b).
