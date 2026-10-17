+++
id = "br-cdw2"
title = "session restart: the pane-gone fallback opens a shell window first (tmux-shell-first), follow-up to 5j35"
kind = "fix"
state = "built"
created_at = "2026-10-17T20:18:12.141Z"
updated_at = "2026-10-17T22:31:39.778714Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:wcdw2",
    "agent:wcdw2b",
]
branch = "bridle/wcdw2b"
commit = "faaea54f4075e4d85986058d2e8b1af3ebe13d7a"
summary = "Remainder of cdw2: bridle up (orchestrator) and bridle advisor start (window and split) now create the tmux window/pane with -P -F #{pane_id} and no command, then send-keys the launcher into it (rule tmux-shell-first). Tests: up.rs fake-tmux asserts send-keys and no bound command; session_test.rs advisor tests updated; new restart test for the pane-gone fallback (new-window -d -P before send-keys -t %8, bare shell window); is_recent_note extracted in session.rs with a 120 s boundary unit test. CHANGELOG line added. The 120 s window is tested at the predicate, not end to end."
ticket = "cdw2"
+++

docs/tickets/open/session-restart-the-pane-gone-fallback-opens-a-shell-window-cdw2.md

## Thread

### note · external:orchestrator · 2026-10-17T20:18:41.033Z
orchestrator: committed (9dc4adc0, rides manager-2's next push) and readied per ready-without-asking (fix). Cherry-pick fbd96e5a as the ask says.

### note · external:advisor/product-manager · 2026-10-17T20:19:00.140Z
advisor (product-manager), verifying 5j35: main (bc988fc3) still has the bound form at crates/bridle/src/session.rs:691 ('tmux new-window -d -n <name> <cmd>'). The worker's fix for it, fbd96e5a (new-window -d -P, then send-keys), is NOT on main: reuse it. Same rule break elsewhere, in scope here since it is the cause the human named on 5j35 ('orchestrator launched that session and keeps using a tmux command that binds the pane'): crates/bridle/src/up.rs:68-73, crates/bridle/src/advisor.rs:96 (tmux_args; tests at 134, 138), tests/session_test.rs:247 asserts 'new-window -d bridle session advisor ...'. Make every session window a shell first, then send-keys. Also untested from 5j35: the 120 s recent-note window (session.rs:712) and the fallback itself; add tests.

### note · agent:wcdw2 · 2026-10-17T20:53:40.509Z
built: dadceec0 (branch bridle/wcdw2)

### note · agent:wcdw2 · 2026-10-17T20:53:43.248Z
done: fbd96e5a cherry-picked cleanly as dadceec0 (new-window -d -P, then send-keys into it); just check exit 0, 1828 run / 1828 passed / 7 skipped; summary and built set. Not done, per the advisor's thread note: up.rs:68-73, advisor.rs:96 (+tests 134/138) and session_test.rs:247 still use the bound new-window form, and the fallback and 120 s note window are untested. Say if you want those in this ticket; I stopped at the cherry-pick as asked. dadceec0

### note · agent:manager-2 · 2026-10-17T21:19:13.981Z
integrated: d88f13215bd8df8ce5a1bcf565b7752586dde095 (branch bridle/wcdw2)

### note · agent:manager-2 · 2026-10-17T21:19:16.356Z
cleanup: removed nothing; kept agent wcdw2 (background job pid 65724)

### note · agent:pm-1 · 2026-10-17T21:19:24.377Z
Remainder brief (pm-1), from manager-2 relaying the advisor's note. Model: Sonnet (small). The cherry-pick (fbd96e5a) is landed; do the rest, no redo:
(1) Make every session window a shell first, then send-keys the command, at the other bound new-window sites: crates/bridle/src/up.rs:68-73, crates/bridle/src/advisor.rs:96 (and its tests ~134/138), crates/bridle/tests/session_test.rs:247. Reuse the helper/approach from fbd96e5a.
(2) Add tests in session.rs for the pane-gone fallback and the 120 s recent-note window.
Acceptance: just check. CHANGELOG line if user-visible. Out of scope: any other session-restart behaviour.

### note · agent:wcdw2b · 2026-10-17T22:31:39.447Z
built: faaea54f4075e4d85986058d2e8b1af3ebe13d7a (branch bridle/wcdw2b)

### note · agent:wcdw2b · 2026-10-17T22:31:39.778Z
done: up.rs/advisor.rs/session_test shell-first, tests for pane-gone fallback and 120s window, CHANGELOG; just check exit 0, 1831 run/1831 passed/7 skipped; main merged; faaea54f4075e4d85986058d2e8b1af3ebe13d7a
