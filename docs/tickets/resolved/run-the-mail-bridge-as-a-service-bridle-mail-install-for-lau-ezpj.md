---
id: ezpj
title: "Run the mail bridge as a service: 'bridle mail install' for launchd (macOS) and systemd (Linux)"
kind: feature
opened: 2026-10-08
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [rs7p, gdyy, 843g]
tasks: [br-ezpj]
closed: 2026-10-09T23:11:02Z
---

## The ask

The human, 2026-10-07 ~9:05 PM ET, via aide: "let's implement the launchd service and linux support for the mail daemon tonight. No reason to wait IMO."

Today `bridle mail run` (`docs/design/mail.md`) runs only by hand: "started by hand or by a service the human sets up". On dalek it runs in a terminal since tonight (rs7p setup done, round trip tested), so it stops when that window closes or the machine reboots.

The ask, modelled on `bridle gateway install` and the daemon's `launchd install` / `systemd install` (`crates/bridle/src/launchd.rs`, `systemd.rs`; `docs/design/cli.md`):
- `bridle mail install [--project P] [--force]` writes a LaunchAgent plist (macOS) or a systemd user unit (Linux) that runs `bridle mail run` for the project, at login/boot, restarting on a crash, logging to a file under `~/.bridle/` (one per project: the bridge serves one project).
- It runs as `external:mail` (`BRIDLE_AS=mail`, token from `credentials.toml`), in the project's workspace or with `--project`, and with `HOME` set so the standard AWS chain finds `~/.aws/credentials`.
- Like the others: prints the load/enable commands (and the `loginctl enable-linger` one on Linux), never runs them; refuses to overwrite without `--force`; refuses with a clear message when `~/.bridle/config.toml` has no `[mail]` or the `mail` token is missing.
- Docs: `docs/design/mail.md` (how it runs), `docs/design/cli.md`.
- Out of scope: the logging fix itself (br-gdyy); the service should log whatever the bridge prints once that lands.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
