+++
id = "br-mfcj"
title = "Mail bridge hangs for good after one stalled S3 download (minimum throughput 0 B/s)"
kind = "bug"
state = "pending"
created_at = "2026-10-15T21:57:45.166Z"
updated_at = "2026-10-15T21:57:57.848885Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
ticket = "mfcj"
+++

Reported by meta-notes' aide on vesta, 2026-10-15, after the human's email to dotfiles-local did not arrive.

On 2026-10-14 ~17:07 UTC, during a network outage on vesta, three of four mail bridges (meta-notes, meta-notes-ui, dotfiles-local) each logged one warning, "streaming error: minimum throughput was specified at 1 B/s, but throughput of 0 B/s was observed", and then nothing for 29 hours. The processes stayed active (running) with an ESTAB connection to S3 but stopped polling: no credential reloads, digests or deliveries. systemd Restart=on-failure never fired because nothing exited. The human's email to dotfiles-local@ sat in inbound/ until the aide restarted the three bridges.

Ask:
- A stalled or failed S3 read must not wedge the poll loop: time out the whole GET and list, and carry on to the next poll.
- A watchdog: when the last successful poll is older than N x poll_secs, raise an incident on the project's daemon (or exit nonzero so systemd restarts it). A silent bridge looks the same as no mail.

## Thread

### note · external:aide@vesta · 2026-10-15T21:57:45.166Z
submitted by external:aide@vesta

### note · agent:pm-1 · 2026-10-15T21:57:57.848Z
Brief (pm-1). Model: Sonnet. Size: S/M. Mail loss that nobody sees, on the human's real projects: rank high. Files: crates/bridle-mail/src/store.rs (~25 client setup, ~66 get_object; also the list call), bridge.rs (the poll loop, ~244-255 for the failure path), failures.rs. Do: (1) wrap each S3 list and each get_object (including reading the body stream to the end) in tokio::time::timeout (say 60 s; a config value only if one already exists for poll settings) so a stalled stream returns an error, is logged by the existing failure path and the loop moves on to the next poll; also set the AWS SDK stalled-stream/operation timeouts in the client config if simple. (2) Watchdog: track the time of the last successful poll; if it is older than 5 x poll_secs, exit with a nonzero status so systemd/launchd restarts the bridge (simplest, no new incident plumbing), after logging why. Check the unit files bridle installs for the mail bridge use Restart=on-failure (or always) and fix them if not; launchd KeepAlive likewise. Tests: a fake store whose get hangs forever times out and the next poll runs; the watchdog trips when polls stop. Docs: mail bridge design doc, CHANGELOG. Acceptance: just check. Out of scope: raising an incident (br-bc37 adds failed-digest events in the same crate; if bc37 has landed, you may reuse its mechanism for a one-line notice, not required). Run after or before br-bc37 (same crate, bridge.rs): a blocked-by edge is set so they run in order. Existing units on other machines are replaced when the bridge is reinstalled/upgraded; say in the done note whether a unit change needs `bridle mail install` re-run, and in the CHANGELOG.
