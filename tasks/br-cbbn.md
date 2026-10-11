+++
id = "br-cbbn"
title = "Scheduled nightly restart of an interactive session at a clock time (e.g. 3 AM)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T10:25:03.871Z"
updated_at = "2026-10-11T03:39:46.773428Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
ticket = "cbbn"
+++

Ticket: docs/tickets/open/scheduled-nightly-restart-of-an-interactive-session-at-a-clo-cbbn.md. Read its "Decided" section (the human, 2026-10-10 and 10-11); it supersedes the older text above it. No dependencies: 4s3z and gq9r have landed, ft3b is not needed.

Goal: opt-in nightly forced restart of an interactive session at a clock time. Config in ~/.bridle/config.toml, machine-wide, one entry per session:
[[nightly_restart]]
project = "notes"
agent = "advisor"     (a role, or advisor/<name> for a named one)
at_time = "4:00 am"   (accept "4:00 am" and "04:00"; machine local time)

Build: a small scheduler component in the daemon that reads this config and fires at at_time, then runs the normal handover. The daemon of that project acts on its own entries. At the time it does what `bridle session restart <role>` does with --handover (the default): asks the session for its handover note, waits up to the handover deadline, restarts the session in its pane; the new session opens with the note. The handover request must say why, e.g. "The human has configured your session to hand over at 4:00 AM every day; it is time to hand over. Write the handover note ..." (use the configured time as written, then the usual handover instructions). Forced: no deferral when the human is active; if no note arrives by the deadline, restart anyway and say so in the daemon log and the morning list. Once per day; a daemon that was down at the time does not catch up. No entry means nothing happens. The orchestrator keeps its own max_uptime.

Files likely: crates/bridle-daemon/src/config.rs (new entry type, parse both time formats, validate), crates/bridle-daemon/src/sessions.rs (and crates/bridle/src/session.rs: find where the restart-with-handover logic lives; reuse it, move it into the daemon if it is CLI-only, do not duplicate; the handover request text needs a reason parameter), a small daemon ticker with an injectable clock, docs/design/agent-host/ (the config and sessions docs), CHANGELOG. Check how the daemon already wakes on time (schedules, br-9xze) and reuse that loop if it fits.

Migration: new optional config key, no migration; say so in the done note.
Acceptance: just check passes, plus a daemon test with a fake clock: fires once at the time, not again that day, restarts after the deadline when no note comes, does nothing with no entry, both time formats parse, bad time is a config error, the handover request contains the reason and the time.
Model: Sonnet. Out of scope: per-role handover instructions (ft3b), deferring when active, a scheduler restart action, catch-up after downtime.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:25:03.871Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T10:25:24.220Z
Triage (pm-1): accept (the human's own ask). Ticket minted (uncommitted; commit on main). Stays pending until approved with `bridle task ready br-cbbn`; then I plan it with edges on br-4s3z, br-gq9r and br-ft3b.

### note · external:advisor/product-manager · 2026-10-09T11:04:38.716Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T03:36:21.718Z
advisor (product-manager): readied. The human decided the design 2026-10-10 ~11:55 PM ET: see the 'Decided' section in ticket cbbn (machine-wide [[nightly_restart]] entries with project, agent, at_time; forced handover then restart; no ft3b dependency). The task body's older brief is superseded where they differ.

### note · external:advisor/product-manager · 2026-10-11T03:37:08.723Z
advisor (product-manager): the human added (2026-10-11 ~12:05 AM ET), now in ticket cbbn's Decided section: the build is a small scheduler component in the daemon that reads the config and fires at at_time, then the normal handover; the handover request must say why, e.g. 'The human has configured your session to hand over at 4:00 AM every day; it is time to hand over. Write the handover note ...'. Fold into the plan if not already there.

### note · external:advisor/product-manager · 2026-10-11T03:39:19.505Z
advisor (product-manager): the human added (2026-10-11 ~12:15 AM ET), in ticket cbbn: skip a session that isn't running; write an event every firing: session.restart.completed (note true/false) or session.restart.skipped (reason not running). Config key names may still change (proposal with the human: [[restart]] project/session/at); hold building the config parsing until confirmed.

### note · external:advisor/product-manager · 2026-10-11T03:39:46.773Z
advisor (product-manager): config names confirmed by the human (2026-10-11 ~12:20 AM ET), in ticket cbbn: [[restart]] in ~/.bridle/config.toml with project, session, at = "HH:MM" (host-local), days (optional, default "all" or a list), reusing the [[focus]]/[[budget.schedule]] time and days parsing. The hold on config parsing is lifted.
