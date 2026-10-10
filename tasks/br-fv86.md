+++
id = "br-fv86"
title = "Install bridle from a release with no clone: one step that downloads, verifies and installs the binary"
kind = "feature"
state = "integrated"
created_at = "2026-10-10T17:17:50.269Z"
updated_at = "2026-10-10T19:45:33.173898Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
priority = "high"
priority_at = "2026-10-10T17:17:50.402293Z"
branch = "bridle/wfv86"
commit = "95294905767268727c042e72a236fa4855d14016"
summary = "Added scripts/install-release.sh [tag]: downloads bridle-TAG-TARGET.tar.gz + SHA256SUMS from the GitHub release (same names/targets as release.rs; default repo jonathanbranam/bridle, env overrides), refuses on checksum mismatch, installs to ~/.local/bin, warns if not on PATH, prints next step. scripts/test-install-release.sh (offline fixture; wired as 'just install-script-test' into 'just check') checks install and bad-checksum rejection. add-a-machine and WSL2 guides use it and set self_upgrade=release; macOS firewall/signing note added; docs/README Releases and CHANGELOG updated. Caveat: bridle doctor on a true no-clone machine and the script on WSL2/real GitHub were not run (doctor reads only the project clone and ~/.bridle/config.toml); docs say so."
ticket = "fv86"
+++

Ticket: docs/tickets/open/install-bridle-from-a-release-with-no-clone-one-step-that-do-fv86.md (read it, with the Releases section of docs/README.md and docs/design/agent-host/daemon.md on self_upgrade). Goal: the first install without a clone. (1) A small script, scripts/install-release.sh (bash, portable to macOS and Linux, no new tools beyond curl/shasum/tar), that takes an optional tag (default newest), downloads bridle-TAG-TARGET.tar.gz for this platform plus SHA256SUMS from the GitHub release (repo from the same source the daemon release code uses, crates/bridle-daemon/src/release.rs), verifies the checksum and refuses on mismatch, installs bridle on the PATH (~/.local/bin, creating it and saying if it is not on PATH), and prints the next step. Reuse the daemon release logic's naming, do not invent a second scheme. A test for the target-name and checksum logic if it can be done offline (a local tarball and SHA256SUMS), else document a manual check. (2) docs/context/add-a-machine.md and the WSL2 guide (grep wsl2 in docs/context) install with it and set [daemon] self_upgrade = release from the start; confirm bridle doctor passes on a machine with no clone and say what it prints. (3) A short macOS note: signing and the firewall prompt with a release binary (CLAUDE.md Build configuration, p88z). Files: scripts/install-release.sh (new), the two guides, docs/README.md Releases, CHANGELOG. Migration: none; existing machines installed by just install are untouched. Acceptance: just check passes; the script run against a local fixture release succeeds and rejects a bad checksum. Model: Sonnet. Out of scope: the UI, project setup and sync (rjd5), changing the release workflow.

## Thread

### note · external:advisor/product-manager · 2026-10-10T17:17:50.402Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-10T17:18:21.778Z
advisor/product-manager (PdM): readied, high, in epic 1 (machine setup, phase 1): the human, 2026-10-10 ~2:00 PM ET, expects to "install bridle without doing a clone, and that can run and self-upgrade" (ticket fv86). Order: after br-751e lands (v0.6.0 can go without it; the guides then use it for the PC), ahead of messaging. Small: docs plus maybe a short script; read the ticket.

### note · agent:wfv86 · 2026-10-10T19:45:02.023Z
done: install-release.sh + offline test + guides/docs; just check exit 0, 1455 tests; 71a972db

### note · agent:manager-2 · 2026-10-10T19:45:08.538Z
integrated: 95294905767268727c042e72a236fa4855d14016 (branch bridle/wfv86)

### note · agent:manager-2 · 2026-10-10T19:45:13.658Z
cleanup: removed nothing; kept agent wfv86 (background job pid 57018)

### note · external:advisor/product-manager · 2026-10-10T19:45:33.173Z
advisor/product-manager (PdM): landing read. Items 1 and 3 met. Not yet verified: item 2's 'bridle doctor passes on a machine with no clone', and the script against real GitHub on WSL2 (the worker says so). Also: the guides put self_upgrade = "release" in ~/.bridle/config.toml, which the daemon only reads once br-36dp lands. So ticket fv86 (and mrhe) stay open until the PC is set up from v0.6.0 with this script, with br-36dp in, and doctor passes there: that is machine-setup phase 1's done-when. Whoever sets up the PC (advisor/wsl2 with the human): please note the result here.
