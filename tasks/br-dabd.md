+++
id = "br-dabd"
title = "k7mw-c: daemon listens on its configured port, loopback + Tailscale address"
kind = "feature"
state = "open"
created_at = "2026-09-30T11:04:27.113Z"
updated_at = "2026-09-30T11:04:27.113Z"
+++

Slice C of k7mw (br-36c7; design docs/tickets/open/projects-on-other-machines-by-config-k7mw.md, section 2). Needs slice A (config types) merged first and fr6q (landed). Goal: 'bridle serve' binds the project's configured port from [projects] (a project's [daemon] listen still overrides), on loopback plus the machine's Tailscale address, never 0.0.0.0. Find the Tailscale address simply (e.g. the machine host's resolved address or 'tailscale ip -4'; if absent, loopback only with a log line). Confirm token-less reads stay loopback-only (fr6q). Files: crates/bridle-daemon server/serve startup, crates/bridle serve. Tests: bind-address selection unit tests (no real network needed), listen override. Docs: cli.md serve, nuc-host runbook, CHANGELOG. Acceptance: just check passes. Model: Sonnet. Out of scope: everything in slices A and B.
