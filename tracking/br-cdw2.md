+++
id = "br-cdw2"
title = "session restart: the pane-gone fallback opens a shell window first (tmux-shell-first), follow-up to 5j35"
kind = "fix"
state = "planned"
created_at = "2026-10-17T20:18:12.141Z"
updated_at = "2026-10-17T21:19:25.446525Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:wcdw2",
]
branch = "bridle/wcdw2"
commit = "d88f13215bd8df8ce5a1bcf565b7752586dde095"
summary = """
Cherry-picked fbd96e5a onto bridle/wcdw2 as dadceec0 (no conflicts). The pane-gone fallback in restart_session (crates/bridle/src/session.rs ~line 689) now runs `tmux new-window -d -P -F #{pane_id} -n <name>` to open a shell window first, then `send-keys -t <pane> <cmd> Enter` to type the relaunch command into it (rule tmux-shell-first). Previously the launcher ran as the window's own command, so it closed with the session. No CHANGELOG change: the br-5j35 line already says the launcher starts in a new tmux window.
Not done here (flagged by the advisor on this thread, left for manager-2 to decide): the same bound form remains at crates/bridle/src/up.rs:68-73 (`new-window -d` with the command), crates/bridle/src/advisor.rs:96 (tmux_args, tests at 134/138) and tests/session_test.rs:247 (asserts 'new-window -d bridle session advisor ...'). The fallback path itself and the 120 s recent-note window (session.rs ~712) also still lack tests.
Check: just check exit 0; 1828 tests run, 1828 passed, 7 skipped (last full count 1828)."""
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
