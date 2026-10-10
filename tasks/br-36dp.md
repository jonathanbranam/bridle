+++
id = "br-36dp"
title = "Machine-wide self_upgrade: let ~/.bridle/config.toml set [daemon] self_upgrade for every daemon on the machine"
kind = "feature"
state = "planned"
created_at = "2026-10-10T18:19:39.799Z"
updated_at = "2026-10-10T18:19:50.959675Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-10T18:19:39.887529Z"
ticket = "36dp"
+++

Ticket: docs/tickets/open/machine-wide-self-upgrade-let-bridle-config-toml-set-daemon-36dp.md (read it). Goal: ~/.bridle/config.toml may set [daemon] self_upgrade, release_repo and self_upgrade_min_interval; the machine value wins over the project's, the same way workflow does; unset on the machine means the project's value applies as today (dalek's bridle daemon keeps building main). Files: crates/bridle-daemon/src/config.rs (Config::load_with_home: extend the machine-file struct and the merge; see how workflow is merged), docs/design/agent-host/roles-and-config.md, docs/design/agent-host/daemon.md (Release upgrade), CHANGELOG. Note the interaction with br-751e (workflow_checkout is used only for self_upgrade = release with no explicit workflow): make sure a machine-set release mode triggers it. Acceptance: just check passes; config tests: machine value wins over project, unset machine falls back to project, release_repo and min interval likewise, invalid value gives the existing error. Migration: none; additive, optional keys; no project files change (that is the point: rule existing-projects). State it in the done note. Model: Sonnet. Out of scope: any other [daemon] keys, editing projects' config files.

## Thread

### note · external:advisor/product-manager · 2026-10-10T18:19:39.887Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-10T18:19:39.937Z
advisor/product-manager (PdM): scheduled in epic machine-setup (phase 1): it blocks item 3, the NUC's daemons on self_upgrade = "release", without committing to the human's existing projects (rule existing-projects). Proposal as the ticket says: ~/.bridle/config.toml may set [daemon] self_upgrade, release_repo and self_upgrade_min_interval; the machine value wins, like workflow. Unset on the machine: the project's value applies as now (dalek's bridle daemon keeps building main). Docs in roles-and-config.md and daemon.md. High: phase 1 is the top epic. pm-1: plan it (small).
