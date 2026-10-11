+++
id = "br-ydyk"
title = "ticket new in another project can write the wrong id prefix (br- in bridle-ui)"
kind = "bug"
state = "claimed"
created_at = "2026-10-11T13:39:02.299Z"
updated_at = "2026-10-11T19:50:12.386Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wydyk",
]
ticket = "ydyk"
+++

Ticket: docs/tickets/open/ticket-new-in-another-project-can-write-the-wrong-id-prefix-ydyk.md (read it and the orchestrator's confirmed cause on the task thread; human approved 2026-10-11).

Confirmed cause: bridle-ui's daemon was on v0.5.0, whose status has no task_prefix, so own_prefix (crates/bridle/src/ticket.rs) falls back to default_task_prefix(project name) = br and ignores [tasks] prefix = "ui" in that project's .bridle/config.toml. Goal: the fallback must not guess. When the daemon does not report a prefix, read the project's [tasks] prefix from its .bridle/config.toml (use the config loader the CLI already has), and if neither source yields one, refuse with a clear error instead of defaulting. The file's prefix and the row's prefix must come from the same source, and `ticket new` must refuse when they disagree.
Files: crates/bridle/src/ticket.rs (own_prefix and ticket new), tests, CHANGELOG.
Migration: none. Existing bad files (id: br-tecf, br-utp5 in bridle-ui) are bridle-ui's to fix; do not touch that repo; mention them in the done note.
Acceptance: just check passes; tests: a daemon status without task_prefix plus a config prefix "ui" writes ui-; no daemon prefix and no config refuses; a prefix mismatch between file and row refuses; ticket new run against another project writes that project's prefix. Model: Sonnet. Out of scope: restarting daemons, repairing other projects' tickets.

## Thread

### note · external:orchestrator · 2026-10-11T13:40:17.606Z
Cause confirmed (orchestrator): bridle-ui's daemon is still on v0.5.0 (started 2026-10-07), whose status has no task_prefix, so own_prefix falls back to default_task_prefix(project name) = br and ignores [tasks] prefix = "ui" in .bridle/config.toml. Fix: the fallback should read the project's [tasks] prefix from its config (or refuse rather than guess). Restarting bridle-ui's daemon onto 0.6.1 removes the trigger there; the fallback bug stays for any older daemon.

### note · external:advisor/product-manager · 2026-10-11T17:18:05.346Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied, normal.
