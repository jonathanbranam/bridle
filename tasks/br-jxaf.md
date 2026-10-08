+++
id = "br-jxaf"
title = "Daemon re-checks Tailscale after start so a boot-time race doesn't leave it loopback-only (v7ug audit 1)"
kind = "bug"
state = "pending"
created_at = "2026-10-08T00:37:10.183Z"
updated_at = "2026-10-08T00:37:10.183Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
+++

Ticket: docs/tickets/open/run-bridle-s-heavy-work-on-the-windows-pc-under-wsl2-v7ug.md, section "Audit", follow-up 1 (read the Tailscale row of the table).
Problem: `serve` reads `tailscale ip -4` once at start (crates/bridle-daemon/src/lib.rs, around lines 343 and 480-500). If the daemon starts before Tailscale is up (boot, or after a WSL restart), it logs at info and binds loopback only, and stays that way: other machines cannot reach it until someone restarts it.
Fix: when a port is configured for the Tailscale listener but no Tailscale address was found at start, retry for a bounded time (e.g. every 5 s up to a few minutes) and bind the Tailscale address as soon as `tailscale ip -4` returns one; log at warn when it gives up. Reuse the existing bind code; do not change behaviour when Tailscale is not configured at all. A fake `tailscale` on PATH makes it testable (see existing tests that fake it, grep `tailscale` in crates/bridle-daemon/tests).
Docs: docs/design/agent-host/daemon.md (listening section), CHANGELOG.
Out of scope: Windows-side Tailscale; the WSL2 setup guide (br-at2j); other hosts' networking.
Migration: none (daemon behaviour only; takes effect on daemon upgrade).
Acceptance: just check passes; test: daemon started with no Tailscale address, address appears later, daemon becomes reachable on it without a restart; test: it gives up and keeps serving loopback. Model: Sonnet.
