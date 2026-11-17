---
id: br-u974
title: Daemon keeps routing orchestrator mail to a project whose waiter exited
kind: bug
created: 2026-11-17
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Orchestrator mail must not go to a daemon whose waiter has exited, and mail that lands where no waiter will see it must say so.

Seen on remus, bridle 0.8.0, 2026-11-16 (orchestrator@remus; home remus-home): a waiter on the flume daemon (`bridle --project flume agent wake external:orchestrator`) exited at ~03:31 UTC with a message. After that the orchestrator waited only on remus-home, but `bridle status` on all three local daemons printed "home remus-home; waiter on flume" for 2h+. Mail to external:orchestrator went to flume and sat "arrived", unread (aide@bridle's reply m-0010 answering o-0053; the flume aide's test m-0009). Restarting the remus-home waiter fixed it.

## Expected

1. When a waiter exits, routing falls back to home (or the newest open waiter); mail never goes to a daemon with no open waiter for that recipient. Find where "waiter on <project>" is recorded (crates/bridle-daemon/src/wake.rs, orchestrator.rs; read `bridle status` to see what prints it) and why exit does not clear it.
2. Mail that lands where no waiter will see it raises something (a wake on home or a status warning), not silence.

## Also check (report on the thread; fix only if small and the same cause)

- A `bridle token pair` run from the flume clone was seen running `pair-mint human --for remus --home flume`. Does pairing take the home from the cwd project rather than the machine's <machine>-home? (crates/bridle/src/token_pair.rs.) A rerun from remus-home left every local daemon at "home remus-home".
- Related reports: aide@bridle's m-0010 (br-gr3v, br-nncj): a daemon created after the last `bridle token pair` has no home-recording token and forwards nothing, with no warning. File a separate ticket for that if it is not the same cause; do not widen this one.

No project-file change expected, so no migration; if the fix changes persisted state, say how it is cleaned up. Docs: docs/design/agent-host/ where waiters and routing are described.

## Acceptance

Tests: waiter exits -> routing returns to home; mail to a recipient with no open waiter anywhere surfaces a warning; normal routing unchanged. `just check` with BRIDLE_* env unset. Model: Sonnet.

Touches token_pair.rs only if the home-from-cwd finding needs a fix; otherwise a different area than br-4wu6.
