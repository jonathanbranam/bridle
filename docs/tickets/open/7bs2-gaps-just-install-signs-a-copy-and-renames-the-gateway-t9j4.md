---
id: br-t9j4
title: "7bs2 gaps: just install signs a copy and renames; the gateway checks its own tailnet listener"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Follow-up to br-7bs2 (landed; PdM verification 2026-10-16). Read the 7bs2 thread and the firewall paragraph in CLAUDE.md.

(a) `just install` (justfile:146-148) still runs cargo install then signs, not `signing::install` (sign a copy named `bridle`, rename into place), so the live binary is briefly linker-signed. Make it use signing::install.
(b) The reachability check runs only in the daemon (crates/bridle-daemon lib.rs around 699 and 1358) and probes the daemon's own address. The gateway is the process actually blocked by the firewall and has no check. Make the gateway check its own tailnet listener after start and raise the same incident when unreachable.

Acceptance: just check; tests for (b) with a fake unreachable listener. Docs: keep cli.md/agent-host docs in step. Model: Sonnet. Out of scope: disabling the firewall (never).
