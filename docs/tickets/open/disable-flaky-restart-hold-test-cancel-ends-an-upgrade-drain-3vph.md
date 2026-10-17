---
id: br-3vph
title: Disable flaky restart_hold_test cancel_ends_an_upgrade_drain_quietly (#[ignore], br-6byg keeps the fix)
kind: chore
created: 2026-10-17
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Per the human's flaky-test rule (2026-10-11) via orchestrator@dalek (br-6byg thread): add #[ignore = "flaky, br-6byg"] to the test. The real fix stays on br-6byg. Branch bridle/ignore6byg, commit 93f3c463.
