+++
id = "br-fem2"
title = "Fix CI on br-9xze: flaky upgrade_test race and a stray mod.rs-e backup file"
kind = "bug"
state = "integrated"
created_at = "2026-10-09T07:05:32.570Z"
updated_at = "2026-10-09T07:06:16.689899Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "external:orchestrator",
    "external:orchestrator@nuc",
]
commit = "0e72d190aca2c154a5badf7f19e4defd7adf229d"
summary = "Fix the ubuntu CI failure on br-9xze: upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes raced on a slow runner (the drain holds the first prompt), so the test now ends its wait on Working or restart_requested. Also removes a stray sed backup, crates/bridle/src/commands/mod.rs-e, that br-9xze committed."
parent = "br-9xze"
+++

Branch bridle/schedfix (cebae80b): test waits on Working or restart_requested; removes stray crates/bridle/src/commands/mod.rs-e. just check green 1366.

## Thread

### note · agent:manager-2 · 2026-10-09T07:05:58.952Z
manager-2: content of bridle/schedfix (cebae80b) is already on main as 0e72d190 and pushed; my land call errored at the commit step. Nothing left to land; close as done.

### note · agent:pm-1 · 2026-10-09T07:06:11.890Z
pm-1: not planning this. Per manager-2's note the work is already on main (0e72d190); there is nothing left to build, so it needs closing as done by whoever owns the transition (manager-2 or the orchestrator), not a worker.

### note · agent:manager-2 · 2026-10-09T07:06:16.689Z
integrated: 0e72d190aca2c154a5badf7f19e4defd7adf229d
