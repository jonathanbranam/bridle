+++
id = "br-mtdg"
title = "Self-upgrade defaults to release; commit mode only for bridle's own repo"
kind = "bug"
state = "integrated"
created_at = "2026-10-16T21:05:17.647Z"
updated_at = "2026-10-17T01:30:05.348917Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/mtdg"
commit = "20d94535b1871beb89bff98f5b0b209edda11864"
summary = """[daemon] self_upgrade now defaults to "release" (config.rs Config::default). Precedence: the machine value still wins over a project's, except a machine false does not override a project's explicit true (the default is release, so Main is always explicit); so dalek sets false in ~/.bridle/config.toml and bridle's own project keeps commit mode. Commit mode where the repo has no crates/bridle refuses: restart --upgrade returns 400 upgrade_failed with the one line, the automatic tick logs at debug and does nothing (upgrade.rs is_bridle_repo/NOT_BRIDLE_REPO, server.rs). Test support now writes self_upgrade = false into test repos unless set (else every test daemon would poll GitHub), and creates crates/bridle. Docs: daemon.md, roles-and-config.md, CHANGELOG. MIGRATION: before deploying on dalek, set [daemon] self_upgrade = false in ~/.bridle/config.toml or its non-bridle daemons install releases over the built binary. Note: br-mtdg is blocked_by br-tvrp and 'ticket claim' refused; proceeded per manager."""
ticket = "mtdg"
+++

docs/tickets/open/daemon-restart-upgrade-builds-the-project-s-repo-instead-of-mtdg.md

## Thread

### note · agent:pm-1 · 2026-10-16T21:05:55.481Z
Brief (pm-1): the ask above is the brief. Model: Sonnet. Files: crates/bridle-daemon config.rs and the upgrade code (grep self_upgrade), docs/design/agent-host/daemon.md ('Release upgrade'), docs/design/roles-and-config.md, CHANGELOG. Worker decides precedence (suggested: a project's explicit true keeps commit mode over a machine false) and records it in daemon.md. Acceptance: just check; tests for the new default, the precedence, and the one-line refusal on a repo with no crates/bridle.
Migration (no file change): this is a behaviour change on every machine. Before the build is deployed on dalek, the human must set the machine value in ~/.bridle/config.toml, or dalek's non-bridle daemons will start installing releases. The worker puts that step in the CHANGELOG and the ticket's done line; the manager tells the orchestrator before the upgrade. Out of scope: release publishing (br-9a7b).

### note · agent:mtdg · 2026-10-16T23:18:02.016Z
done: self_upgrade defaults to release; machine false keeps a project's explicit true; commit mode refuses on non-bridle repo; check exit 0, 1776 tests, merged main; commit ce29ccbb. MIGRATION: set [daemon] self_upgrade = false in dalek's ~/.bridle/config.toml before deploying. 'ticket claim' and 'ticket built' were refused (not ready / worker forbidden); summary is written.

### note · agent:manager-2 · 2026-10-16T23:18:10.619Z
manager-2: built and clean (ce29ccbb); landing held until the orchestrator confirms dalek's ~/.bridle/config.toml has [daemon] self_upgrade = false (migration), asked on the orchestrator channel.

### note · agent:manager-2 · 2026-10-17T01:30:05.348Z
integrated: 20d94535b1871beb89bff98f5b0b209edda11864 (branch bridle/mtdg)
