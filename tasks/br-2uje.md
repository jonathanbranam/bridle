+++
id = "br-2uje"
title = "bridle doctor: Linux/WSL checks and per-OS fix text (v7ug audit 3)"
kind = "chore"
state = "planned"
created_at = "2026-10-08T00:37:14.776Z"
updated_at = "2026-10-08T00:37:42.911321Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
size = "S"
+++

Ticket: docs/tickets/open/run-bridle-s-heavy-work-on-the-windows-pc-under-wsl2-v7ug.md, section "Audit", follow-up 3.
Goal: `bridle doctor` (crates/bridle/src/doctor.rs; read how its existing checks and the `--strict` flag from br-6hx4 work first) is correct on a Linux/WSL2 host.
1. The Claude login fix text (around doctor.rs:423) only mentions macOS keychain steps. Make it per-OS: on Linux the login lives in ~/.claude and the fix is `claude auth login` (headless URL/device flow); keep the macOS text for macOS.
2. New warnings, Linux only, same warning style and strict behaviour as the existing ones: (a) the workspace path is under /mnt/ (slow, no unix permissions on WSL2): warn, naming the path and saying to use the Linux home; (b) systemd is not PID 1 (read /proc/1/comm; on WSL this means `systemd=true` is missing from /etc/wsl.conf): warn with that fix; (c) linger is off for the user while `bridle systemd install` units exist: warn with `sudo loginctl enable-linger <user>`. Each must be skipped silently on macOS and testable with injected inputs (path, /proc/1/comm contents, linger state) rather than the real machine.
Docs: docs/design/cli.md doctor section, CHANGELOG. Out of scope: any WSL2 setup guide text (br-at2j), the Tailscale retry.
Migration: none. Acceptance: just check passes; tests for each new warning and for the per-OS login text. Model: Sonnet.
