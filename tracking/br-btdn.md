+++
id = "br-btdn"
title = "Daemon restart and self-upgrade blocked forever: manager.spawning() stays true with no agent spawning ('still busy: a spawning agent')"
kind = "bug"
state = "open"
created_at = "2026-10-06T21:56:19.445Z"
updated_at = "2026-10-12T03:14:48.432899Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "btdn"
+++

## The ask

Critical (orchestrator, 2026-10-12): the stuck spawn counter has now stopped bridle's daemon twice.
y455 (2026-10-05/06, ~23 h without restart or self-upgrade) and up5r (2026-10-12 01:56-03:13Z: the
human's `bridle daemon restart` drained for 77 min, every agent message held, until `launchctl
kickstart -k`). Since y455, `perform_restart` drains with no timeout (`server.rs` ~4847), so a
stuck counter now freezes the whole workforce, not just restarts.

The y455 postmortem (docs/tickets/open/incident-bridle-s-daemon-couldn-t-restart-or-self-upgrade-fo-y455.md,
"Recommendations") showed the counter's drop guard releases on every exit path; it sticks only
while a `spawn` future hangs on an unbounded await. Build recommendations 1, 2 and 5:

1. **Name the holder.** Replace the bare `Inner::spawning` counter (supervisor.rs ~191, ~803-838)
   with a small registry (agent name, started-at) behind the same `spawning()`. The drain wake and
   `bridle status --json` name it ("a spawning agent: incident-2y3m, 77 min").
2. **Bound the spawn.** A timeout on the whole `AgentManager::spawn` (15 min) plus
   `kill_on_drop(true)` on the `warm_target` copy, so a hang becomes a logged spawn failure that
   releases the guard.
3. **A drain stuck only on a spawn ends.** If a restart's drain has no busy agent and the only
   holder is a spawn older than the timeout in 2, drop it from the quiet-point check and restart
   (log and event it with the holder's name). Busy agents are still waited for, as now.

Acceptance: a test with a spawn that never finishes shows (a) the holder named in status, (b) the
spawn failing after the (test-shortened) timeout with the guard released, (c) a restart proceeding
once the only holder is past the timeout. `just check` green.

Model: Sonnet (daemon concurrency). Touches crates/bridle-daemon/src/{supervisor.rs,server.rs,
worktree.rs}, bridle-api types.rs if status gains a field, docs/design/agent-host (restart section).

Not yet: postmortem items 3 (upgrade-skipped alert), 4 (restart.refused events) and 6 (CI ssh
key); separate tickets if still wanted. Rejected: a fixed drain timeout that restarts while agents
are mid-turn (cuts turns off, which the no-timeout drain was made to avoid).

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.466Z
watching the task
