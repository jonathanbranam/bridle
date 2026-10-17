+++
id = "br-cdw2"
title = "session restart: the pane-gone fallback opens a shell window first (tmux-shell-first), follow-up to 5j35"
kind = "fix"
state = "planned"
created_at = "2026-10-17T20:18:12.141Z"
updated_at = "2026-10-17T20:32:06.862140Z"
created_by = "agent:manager-2"
watchers = ["agent:manager-2"]
ticket = "cdw2"
+++

docs/tickets/open/session-restart-the-pane-gone-fallback-opens-a-shell-window-cdw2.md

## Thread

### note · external:orchestrator · 2026-10-17T20:18:41.033Z
orchestrator: committed (9dc4adc0, rides manager-2's next push) and readied per ready-without-asking (fix). Cherry-pick fbd96e5a as the ask says.

### note · external:advisor/product-manager · 2026-10-17T20:19:00.140Z
advisor (product-manager), verifying 5j35: main (bc988fc3) still has the bound form at crates/bridle/src/session.rs:691 ('tmux new-window -d -n <name> <cmd>'). The worker's fix for it, fbd96e5a (new-window -d -P, then send-keys), is NOT on main: reuse it. Same rule break elsewhere, in scope here since it is the cause the human named on 5j35 ('orchestrator launched that session and keeps using a tmux command that binds the pane'): crates/bridle/src/up.rs:68-73, crates/bridle/src/advisor.rs:96 (tmux_args; tests at 134, 138), tests/session_test.rs:247 asserts 'new-window -d bridle session advisor ...'. Make every session window a shell first, then send-keys. Also untested from 5j35: the 120 s recent-note window (session.rs:712) and the fallback itself; add tests.
