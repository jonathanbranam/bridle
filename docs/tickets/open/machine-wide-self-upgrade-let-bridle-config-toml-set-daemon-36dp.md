---
id: 36dp
title: "Machine-wide self_upgrade: let ~/.bridle/config.toml set [daemon] self_upgrade for every daemon on the machine"
kind: feature
opened: 2026-10-10
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

## Ask

Whether a daemon builds main or installs releases is a property of the machine (a build host vs. a client), not of the project. Today `[daemon] self_upgrade` is read only from the project's committed `.bridle/config.toml` (`Config::load_with_home`, crates/bridle-daemon/src/config.rs: the machine file supplies only `[budget]` and `workflow`). So putting the NUC's daemons (meta-notes, notes, dotfiles-local, meta-notes-ui) on `self_upgrade = "release"` (machine-setup phase 1, item 3) would mean committing to each of the human's existing projects, and that commit would switch every machine running those projects, not just the NUC.

## Proposal

Read `[daemon] self_upgrade` (and `release_repo`, `self_upgrade_min_interval`) from `~/.bridle/config.toml` too, with the machine value winning over the project's, the same way `workflow` does. Document it in docs/design/agent-host/roles-and-config.md and daemon.md (Release upgrade). Small; Haiku or Sonnet.

## Why not just edit the project configs

The worst case of not doing it: the NUC stays on hand-installed builds from the bridle-work clone, and client-relevant fixes reach it only when someone rebuilds there. Editing project configs instead touches the human's existing projects (rule existing-projects) and leaks a machine choice into every clone.

Found by the orchestrator, 2026-10-10, preparing the NUC switch to release upgrades.
