---
id: br-tvrp
title: "Release-mode self-upgrade does not re-sign: the gateway is firewall-blocked again after a release upgrade on a Mac"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-17T03:21:38Z
---

## The ask

Found by the orchestrator verifying the 10-15/10-16 merges (br-7bs2 gap, confirmed). `release::install` (crates/bridle-daemon/src/release.rs:212) copies to bridle.new and renames, and release.yml has no codesign step. On a Mac with `self_upgrade = "release"` the installed binary keeps its linker identifier, so the gateway, kickstarted onto it, is blocked by the macOS firewall again (see the firewall paragraph in CLAUDE.md, br-7bs2).

Fix: on macOS, call `signing::install` from `release::install`, after the rollback stash, so the installed copy is signed `-i bridle` with the stable identity (ad-hoc when none) and renamed into place. Tests: the call happens on macOS and not elsewhere (gate by cfg or an injected platform flag), and a failed sign rolls back. Docs: daemon.md 'Release upgrade'. Acceptance: just check. Model: Sonnet. Priority high (multi-machine). Out of scope: signing in CI.
