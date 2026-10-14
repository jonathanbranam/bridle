---
id: br-tcae
title: "Mail bridge runs a stale binary after upgrades, and its digest fails silently on new ticket kinds/states: no digest 10-08 to 10-13"
kind: bug
created: 2026-10-14
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-pcde, br-a885]
---

The human, 2026-10-14 ~8:30 AM ET (to aide), verbatim: "I haven't seen a bridle daily digest email since I think 10.7. Can you check on that?"

## What happened (aide's subagent, read-only, 2026-10-14)

- launchd last (re)loaded `dev.bridle.mail.bridle` on 2026-10-07 22:06. That process ran the
  10-07 binary for a week: nothing restarts the mail unit when the binary is upgraded
  (`crates/bridle/src/mail_install.rs` only restarts it on a crash).
- That build predates br-gdyy (738b831e, 10-08, logging in `mail run`), so every failure was
  silent: `~/.bridle/mail-bridle.log` stayed empty.
- The digest (`crates/bridle-mail/src/bridge.rs` `poll_outbound`, ~208-260) aborts on any `?`
  before the send. The old client's strict wire enums don't know ticket kinds `fix`/`epic`/`docs`
  (bpku, 10-11) or states `held`/`built` (stx8, 10-11), so `tasks()` (~bridge.rs:244) failed to
  decode and no digest went out, certainly from 10-11 on (10-08 to 10-10 not provable: no logs).
- Inbound and question mails kept working in that time (e.g. m-8260 on 10-09).
- Fixed by accident on 10-14: the bridge restarted at 14:20 ET (for br-pcde) and mailed a digest
  ("digest mailed", 18:20:38Z); it now runs 0.7.2.

## The ask

1. Self-upgrade and `just install` restart the `dev.bridle.mail.*` units (and systemd's) along
   with the daemon, or the bridge notices its binary was replaced and re-execs.
2. The digest doesn't silently die: unknown enum values are tolerated (or the item skipped), and
   a failed digest is logged and raised (an event or a message to the orchestrator; see br-zpc7).
