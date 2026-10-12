---
id: br-jw75
title: "Move bridle and bridle-ui to remus: remus becomes the dev machine, dalek a release client"
kind: epic
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-kx5u]
related: [br-kx5u, br-3pc2, br-v7ug, br-785a, br-du7e]
---

## The ask

The human, 2026-10-11 ~9:05 PM ET (to advisor product-manager), verbatim:

> we will test and confirm moving projects with something small like data-contracts but eventually bridle needs to be movable; make an epic for moving bridle and bridle-ui to remus wsl2 - that is going to mean disabling auto-builds on remus; release build installs turned off; switch to bridle and bridle-ui local; publishing, etc. then dalek needs to switch to using release builds

## Outcome

remus (the Windows PC, WSL2) becomes bridle's development machine: bridle and bridle-ui are served, built and released from there. dalek becomes a release client like the NUC.

## Done when (PdM draft)

1. bridle-ui, then bridle, have moved dalek -> remus with the epic br-kx5u commands, with the loss list confirmed and nothing silently left behind.
2. remus runs bridle as the dev machine: release installs off (no `self_upgrade = "release"`), bridle and bridle-ui built and installed from the local clones (bridle's own build-from-main self-upgrade or `just install`; the local bridle-ui clone serves the UI), and releases are cut and published from there (tag, `ui.pin` bump, release workflow).
3. dalek runs `self_upgrade = "release"` (binary and `~/.bridle/ui` from releases), with its own orchestrator and its other projects still working.
4. Everything else that today assumes bridle lives on dalek is settled: see br-3pc2 Section 2 (moving bridle itself).

## Open points for the design (first step)

- The PdM reads "disabling auto-builds on remus; release build installs turned off" as: remus stops installing release builds and builds from its local clones instead. To confirm with the human: does "disabling auto-builds" mean the release auto-upgrade, or also the build-from-main self-upgrade?
- Each machine's orchestrator runs in the bridle project today (one per machine). Once bridle is served on remus, where does dalek's orchestrator run, and what supervises it? (br-3pc2: `[orchestrator] enabled` is in bridle's committed config.)
- Workflow source: dalek's other projects read rules and roles from the dalek bridle clone; who keeps it current once dalek stops building bridle?
- Signing (`just sign-setup`, firewall identity) is a dalek concern; on remus (Linux) not needed. Release signing of the macOS tarballs stays in CI.
- Sessions for bridle (aide, advisors, the PdM) must run on remus, where bridle's daemon is (br-3pc2 finding 8).
- bridle-ui dev on remus: the gateway, `~/.bridle/ui` from the local clone (br-785a, dev-machine follow).

## Children (to file when planned)

1. Design the switch (the open points above), then the human's review.
2. remus as dev machine: config and install switch.
3. dalek as release client: config switch, orchestrator and other projects verified.
4. Move bridle-ui, then bridle (runs the kx5u commands; a human to-do with the agents ready).

Blocked by epic br-kx5u (the move commands), proven first on a small project (data-contracts).
