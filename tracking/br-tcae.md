+++
id = "br-tcae"
title = "Mail bridge runs a stale binary after upgrades, and its digest fails silently on new ticket kinds/states: no digest 10-08 to 10-13"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T21:05:25.085Z"
updated_at = "2026-10-15T00:14:52.961342Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:tcae",
]
ticket = "tcae"
+++

docs/tickets/open/mail-bridge-runs-a-stale-binary-after-upgrades-and-its-diges-tcae.md

## Thread

### note · external:aide · 2026-10-14T21:05:32.408Z
From aide, for the human's question ("I haven't seen a bridle daily digest email since I think 10.7. Can you check on that?"): filed br-tcae. The mail bridge ran the 10-07 binary for a week (nothing restarts mail units on upgrade) and its digest died silently on new ticket kinds/states. Digest resumed today after the pcde restart. Please plan; the stale-binary half also affects vesta's bridges.

### note · external:aide · 2026-10-14T22:11:48.035Z
From the human, via aide, verbatim: "yes, please fix the mail daemon it should self-upgrade and restart with the rest of bridle. That should work on the machine that builds bridle and those that install it from release". Recorded on br-tcae as the decision. Please plan it.

### note · agent:pm-1 · 2026-10-14T22:18:19.462Z
Brief (pm-1). Model: Sonnet. Size: M. The ticket body is the spec, with the human's decision (item 1 covers both upgrade paths: source-build self-upgrade and `just install`, and release self-upgrade, on launchd and systemd). Land in order and tell the manager at the first stop that gets long: (A) item 2, small: the digest tolerates unknown ticket kinds/states (crates/bridle-mail/src/bridge.rs `poll_outbound` ~208-260, `tasks()` ~244; the wire enums in crates/bridle-api/src/types.rs: add an Unknown/other fallback with serde(other) or skip the item), and a failed digest is logged and raised once per failure (an event or a message to the orchestrator, per br-zpc7 if it landed; else a log line plus a message, not per poll). Tests: a task list with an unknown kind and state still produces a digest; a failed digest logs. (B) item 1: after a self-upgrade or `just install` installs a new binary, restart the `dev.bridle.mail.*` launchd units and the systemd `bridle-mail-*` units (find mail_install.rs and the self-upgrade/install code: git grep -n "self_upgrade\|kickstart" crates), in the same step as the daemon restart; skip units that are not installed or whose machine has mail off. Docs: docs/design/mail.md, the self-upgrade doc section, CHANGELOG.md. Touches the self-upgrade/restart code that br-cvfu also edits: run after cvfu. Never kill by name (rule no-kill-by-name): use launchctl kickstart / systemctl restart. No migration. Acceptance: just check. Out of scope: re-exec in the bridge.
