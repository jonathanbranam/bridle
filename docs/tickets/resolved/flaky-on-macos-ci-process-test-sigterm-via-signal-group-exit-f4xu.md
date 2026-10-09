---
id: f4xu
title: "Flaky on macOS CI: process_test sigterm_via_signal_group_exits_143 exits 1, not 143"
kind: bug
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-f4xu]
closed: 2026-10-09T23:11:08Z
---

## The ask

Make `crates/bridle-claude/tests/process_test.rs` `sigterm_via_signal_group_exits_143` deterministic on macOS without weakening what it checks (that SIGTERM to the group reaches the fake's handler and it exits 143). Find why the fake exited 1: read `crates/bridle-claude/tests/fake-claude.py`'s SIGTERM handler and what else in the group can get the signal first (the python3/pyenv shim chain that the test's comment already mentions, or the handler racing a read on stdin). Don't add sleeps or retries. Critical: any red CI on main, a flaky test included, is (the human, 2026-10-03).

## What happened

CI run https://github.com/jonathanbranam/bridle/actions/runs/37931990047 on main 8debefe6 (a docs-only commit, 2026-10-09 12:57Z): macOS failed, Linux passed; 1379 of 1380 passed. The one failure:

```
bridle-claude::process_test sigterm_via_signal_group_exits_143
panicked at crates/bridle-claude/tests/process_test.rs:257:5
assertion `left == right` failed
  left: Some(1)
 right: Some(143)
```

The next commit (5bf90cd6) passed on both, so main is green again; this is a flake, not a regression. The test already waits for a control response before signalling, so the fake's interpreter was up; exit code 1 suggests the handler ran into an error (or something else in the group exited first and the fake saw a broken pipe), not that the signal killed it early.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
