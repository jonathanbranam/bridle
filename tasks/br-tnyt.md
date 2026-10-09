+++
id = "br-tnyt"
title = "Machine load notes repeat every few minutes: add a quiet period, and send one note per machine, not one per daemon"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T22:04:06.527Z"
updated_at = "2026-10-09T23:35:39.332758Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
priority = "high"
priority_at = "2026-10-09T22:05:37.529815Z"
branch = "bridle/wtnyt"
commit = "e89d8401a5b2622f53bccd75f517288098cebd07"
summary = "Load notes are rate-limited in load.rs: a note arms again only after load stays under the threshold for [machine] load_quiet_below (10m), and none is sent within load_note_gap (30m) of the last note from any daemon, recorded in ~/.bridle/load-note.stamp (epoch secs, atomic write; missing/garbled = send). A crossing skipped because of the stamp still uses up the arming. Hold, recipient and text unchanged. Docs: operating-model, roles-and-config, CHANGELOG. Tests use an injected clock (tick_at) and a shared temp home. Caveat: settle_wake_test crossing_the_settle_time_notes_the_manager_once failed once under load in the first check run, passed alone and in the final full run."
ticket = "tnyt"
+++

Ticket: docs/tickets/open/machine-load-notes-repeat-every-few-minutes-add-a-quiet-peri-tnyt.md (read it: the human's words, the facts, the ask). Model: Sonnet. The recipient stays the orchestrator (d48r was withdrawn); do not change it.

Goal: the machine load note does not repeat over and over, and a machine-wide condition is reported once per machine, not once per daemon.

Do, all in crates/bridle-daemon/src/load.rs (the watch samples every 30 s and notes on each upward crossing of [machine] load_per_core) and crates/bridle-daemon/src/config.rs:
1. Quiet period (hysteresis plus a minimum gap): after a note, send no new one until the load has stayed below the threshold for a while (default 10 minutes below), and never more than one note per interval (default 30 minutes). Both config values in the [machine] table beside load_per_core, with those defaults, spelled like the neighbouring settings.
2. One note per machine: a machine-wide stamp file under ~/.bridle/ (e.g. load-note.stamp, holding the last-sent time; write atomically; use the home dir the daemon already uses, tests use a temp home) read before sending; the first daemon to cross sends, the others skip while the stamp is inside the minimum gap. A missing or unreadable stamp means send.
3. Unchanged: what the daemon does under load (it holds new spawns), the note's recipient, and the note's text (br-g76s rewrites the text and adds events, the consumers line and escalation after this lands; do not do its work).
4. Docs: docs/design/agent-host/operating-model.md ("Load watch") and daemon.md if it lists [machine] settings; CHANGELOG.md.
Tests with an injected clock and LoadSource (see the existing tests in load.rs): flapping around the threshold sends one note; a second note only after the quiet below-threshold period or the minimum gap, per the rule; a second daemon (second watcher, same temp home) skips inside the gap and sends after it.
Acceptance: just check passes. Migration: none; defaults apply on upgrade (CHANGELOG notes the two new keys).
Out of scope: hold events, bridle-process consumers line, escalation (br-g76s, which runs after this task); the governor threshold.

## Thread

### note · external:advisor/product-manager · 2026-10-09T22:05:37.529Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T22:05:37.577Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T22:05:37.627Z
advisor (product-manager): readied on the human's ask ("tell aide to file a ticket to deal with that", quoted in the ticket). High: every repeat wakes the orchestrator, three copies at a time, which costs its context and tokens all evening. Theme performance, next to br-g76s (same file, load.rs: if g76s is in flight, merge main first). Recipient stays the orchestrator.

### note · agent:wtnyt · 2026-10-09T23:35:31.044Z
done: load note quiet period (10m below) + 30m gap + machine-wide stamp; just check exit 0, 1429 tests, main merged; d3055347

### note · agent:manager-2 · 2026-10-09T23:35:39.332Z
integrated: e89d8401a5b2622f53bccd75f517288098cebd07 (branch bridle/wtnyt)
