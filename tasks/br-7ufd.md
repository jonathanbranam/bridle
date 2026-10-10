+++
id = "br-7ufd"
title = "Self-upgrade at most every few hours, batching the landings; critical fixes go through at once"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T22:02:24.988Z"
updated_at = "2026-10-10T01:32:19.388547Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
priority = "high"
priority_at = "2026-10-09T22:02:57.905342Z"
branch = "bridle/w7ufd"
commit = "41d6e0fd6d7b2fe91cc2b8c819441fd295f250cb"
summary = "Added [daemon] self_upgrade_min_interval (default 3h, 0s = old behaviour). The automatic self-upgrade (server.rs self_upgrade_tick_at) holds a newer green commit, logged once per candidate, until the interval has passed since the newest stored upgrade.built event (upgrade.rs last_built_at; chosen over daemon.started because a crash restart emits that too; survives restarts since it reads the store). Explicit restart --upgrade ignores it; a critical landing does not trigger an upgrade (deferred, stated in daemon.md). Tests: unit (interval, store-derived time) and an integration test across a restart. Docs: daemon.md, roles-and-config.md, CHANGELOG. Note: bridle's own self_upgrade is off pending br-x7fx; tell the human via aide when this lands."
ticket = "7ufd"
+++

Ticket: docs/tickets/open/self-upgrade-at-most-every-few-hours-batching-the-landings-c-7ufd.md (read it: the human's words, the facts, the ask). Model: Sonnet.

Goal: the automatic self-upgrade waits a minimum interval after the last upgrade, then takes the newest green commit, so landings in between go in as one batch. An explicit upgrade still goes through at once.

Do:
1. Config: `[daemon] self_upgrade_min_interval` in crates/bridle-daemon/src/config.rs, a duration in the style of the existing duration settings (check how other intervals are spelled there and reuse). Default "3h". "0" (or the smallest accepted value) restores today's behaviour. Document it where `self_upgrade` is documented (docs/design/agent-host/daemon.md, "Automatic upgrade"; docs/design/config doc if one lists daemon keys).
2. Behaviour: find the CI watcher's upgrade decision (grep self_upgrade and `upgrade.built` in crates/bridle-daemon/src/{server.rs,lib.rs} and the watcher module). The last-upgrade time must survive a daemon restart (the upgrade itself restarts the daemon): derive it from the stored `upgrade.built`/`daemon.started` event timestamps or an existing persisted field, not from memory; pick the smaller and say which in the task comment. While inside the interval, a newer green commit is noted (log line once per candidate, not every tick) and not built. When the interval has passed, build the newest green main commit, as today (docs-only candidates still skipped).
3. Explicit `bridle restart --upgrade` (human or orchestrator) ignores the interval, as today; add a test. Critical-priority task landing triggering it by itself: NOT built (deferred, not yet needed: the orchestrator can run the explicit command; see the ticket's open design point); state that in the daemon.md text.
4. Tests: interval blocks a second automatic upgrade; passes after the interval; explicit upgrade ignores it; state survives a restart (simulate by constructing the watcher from stored events). Use injected time, not sleeps.
5. Docs: daemon.md, CHANGELOG.md. bridle's own .bridle/config.toml needs no edit (default applies).
Acceptance: just check passes. Migration: none; existing projects get the 3 h default on upgrade (CHANGELOG notes this change of behaviour and the config key to restore the old one).
Out of scope: the nightly maintenance window and scheduler (yfv5, 3nyk), session registry survival across restarts (e35h), batching by landing count.

## Thread

### note · external:advisor/product-manager · 2026-10-09T22:02:57.905Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T22:02:57.945Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T22:02:57.980Z
advisor (product-manager): readied on the human's approval of option (1) (quoted in the ticket). Priority high: 17 restarts a day drain every agent and end interactive waits, and the v6kr baseline starts 4:00 AM ET 2026-10-10, so fewer restarts also makes that measurement cleaner. Theme reliability, no epic. Default interval 3 h unless the human says otherwise; keep it a config value.

### note · external:aide · 2026-10-09T22:04:46.377Z
aide: the human turned self_upgrade off meanwhile (uncommitted, in their checkout's .bridle/config.toml; read at the next daemon start). Their reminder to turn it back on when this lands is br-x7fx; whoever lands this, tell the human (via aide).

### note · agent:w7ufd · 2026-10-09T23:48:51.254Z
last-upgrade time = newest stored upgrade.built event (not daemon.started: crashes emit that too). See summary.

### note · agent:w7ufd · 2026-10-10T00:50:03.484Z
done: self_upgrade_min_interval (default 3h; 0s = old behaviour), last upgrade = newest stored upgrade.built event, explicit restart --upgrade ignores it; just check exit 0, 1432 tests passed (one earlier governor_test timeout was load, 17/17 on rerun); c8c73f1c. Tell the human via aide on landing: self_upgrade is off in their checkout (br-x7fx).

### note · agent:manager-2 · 2026-10-10T00:50:09.980Z
main moved; merge main into bridle/w7ufd (just check only if code changed) and message me the new tip.

### note · agent:w7ufd · 2026-10-10T00:50:23.743Z
merged main (docs-only change, br-tnyt already included), no check re-run; new tip 4bcc79e6

### note · agent:manager-2 · 2026-10-10T01:32:02.188Z
integrated: 41d6e0fd6d7b2fe91cc2b8c819441fd295f250cb (branch bridle/w7ufd)

### note · agent:manager-2 · 2026-10-10T01:32:19.388Z
cleanup: removed agent w7ufd, branch bridle/w7ufd
