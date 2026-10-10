+++
id = "br-fv86"
title = "Install bridle from a release with no clone: one step that downloads, verifies and installs the binary"
kind = "feature"
state = "planned"
created_at = "2026-10-10T17:17:50.269Z"
updated_at = "2026-10-10T17:18:31.408927Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-10T17:17:50.402293Z"
ticket = "fv86"
+++

Ticket: docs/tickets/open/install-bridle-from-a-release-with-no-clone-one-step-that-do-fv86.md (read it, with the Releases section of docs/README.md and docs/design/agent-host/daemon.md on self_upgrade). Goal: the first install without a clone. (1) A small script, scripts/install-release.sh (bash, portable to macOS and Linux, no new tools beyond curl/shasum/tar), that takes an optional tag (default newest), downloads bridle-TAG-TARGET.tar.gz for this platform plus SHA256SUMS from the GitHub release (repo from the same source the daemon release code uses, crates/bridle-daemon/src/release.rs), verifies the checksum and refuses on mismatch, installs bridle on the PATH (~/.local/bin, creating it and saying if it is not on PATH), and prints the next step. Reuse the daemon release logic's naming, do not invent a second scheme. A test for the target-name and checksum logic if it can be done offline (a local tarball and SHA256SUMS), else document a manual check. (2) docs/context/add-a-machine.md and the WSL2 guide (grep wsl2 in docs/context) install with it and set [daemon] self_upgrade = release from the start; confirm bridle doctor passes on a machine with no clone and say what it prints. (3) A short macOS note: signing and the firewall prompt with a release binary (CLAUDE.md Build configuration, p88z). Files: scripts/install-release.sh (new), the two guides, docs/README.md Releases, CHANGELOG. Migration: none; existing machines installed by just install are untouched. Acceptance: just check passes; the script run against a local fixture release succeeds and rejects a bad checksum. Model: Sonnet. Out of scope: the UI, project setup and sync (rjd5), changing the release workflow.

## Thread

### note · external:advisor/product-manager · 2026-10-10T17:17:50.402Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-10T17:18:21.778Z
advisor/product-manager (PdM): readied, high, in epic 1 (machine setup, phase 1): the human, 2026-10-10 ~2:00 PM ET, expects to "install bridle without doing a clone, and that can run and self-upgrade" (ticket fv86). Order: after br-751e lands (v0.6.0 can go without it; the guides then use it for the PC), ahead of messaging. Small: docs plus maybe a short script; read the ticket.
