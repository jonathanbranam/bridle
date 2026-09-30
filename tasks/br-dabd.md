+++
id = "br-dabd"
title = "k7mw-c: daemon listens on its configured port, loopback + Tailscale address"
kind = "feature"
state = "integrated"
created_at = "2026-09-30T11:04:27.113Z"
updated_at = "2026-09-30T12:05:35.004801Z"
branch = "bridle/k7mw-c"
commit = "cbaf6c009e3152fd6490b08ed7e1312ea57240aa"
summary = "bridle serve now binds the [projects] port for its project when [machine] name matches, on 127.0.0.1 plus the tailscale ip -4 address (loopback only with a log line if absent), never 0.0.0.0; --listen then [daemon] listen (new Config.listen_set) still override. Daemon serves one axum listener per address (lib.rs bind_addrs/this_machine_port/tailscale_ip, unit-tested). fr6q loopback-only token-less reads unchanged. Docs: cli.md, nuc-host.md, CHANGELOG."
+++

Slice C of k7mw (br-36c7; design docs/tickets/open/projects-on-other-machines-by-config-k7mw.md, section 2). Needs slice A (config types) merged first and fr6q (landed). Goal: 'bridle serve' binds the project's configured port from [projects] (a project's [daemon] listen still overrides), on loopback plus the machine's Tailscale address, never 0.0.0.0. Find the Tailscale address simply (e.g. the machine host's resolved address or 'tailscale ip -4'; if absent, loopback only with a log line). Confirm token-less reads stay loopback-only (fr6q). Files: crates/bridle-daemon server/serve startup, crates/bridle serve. Tests: bind-address selection unit tests (no real network needed), listen override. Docs: cli.md serve, nuc-host runbook, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: everything in slices A and B.

## Thread

### note · agent:k7mw-c · 2026-09-30T12:04:47.902Z
done: serve binds [projects] port on loopback + Tailscale, listen overrides, tests/docs; just check passes (878); 9879d83

### note · agent:manager-2 · 2026-09-30T12:04:52.506Z
integrated: cbaf6c009e3152fd6490b08ed7e1312ea57240aa (branch bridle/k7mw-c)

### note · agent:manager-2 · 2026-09-30T12:05:35.004Z
cleanup: removed agent k7mw-c, branch bridle/k7mw-c
