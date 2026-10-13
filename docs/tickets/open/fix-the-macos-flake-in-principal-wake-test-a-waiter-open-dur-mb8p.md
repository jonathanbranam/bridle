---
id: br-mb8p
title: Fix the macOS flake in principal_wake_test a_waiter_open_during_a_restart (carries br-tmmj)
kind: fix
created: 2026-10-13
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-tmmj]
---

## The ask

Carries br-tmmj, which is stuck in `reopened` (a non-incident reopened ticket can't be planned). The human approved the fix on 2026-10-13. The work and pm-1's brief are on br-tmmj's thread: find why `daemon.stopping` sometimes never arrives within the 60 s guard on macOS, fix the cause (product or test, not a longer timeout), and remove the #[ignore] added in 4849f58b. Acceptance: the 30-run Verify loop passes, the test isn't ignored, `just check` passes.
