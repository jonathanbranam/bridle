---
id: br-xce6
title: "Flaky under load: restart_hold_test (br-cvfu's new test) failed once in eg64's just check"
kind: fix
created: 2026-10-15
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

## What happened

2026-10-15 ~02:20Z, during br-eg64's `just check` with the machine at ~3.4 load per core, a test in `restart_hold_test` (added by br-cvfu, `bridle daemon restart --cancel/--postpone`) failed. It passed alone and on a rerun. Reported by manager-2 (m-13345); the failing test name and error weren't recorded, so check eg64's worker log or rerun under load.

## Ask

Under the flaky-test rule (the human, 2026-10-11): ignore it, or mark it `#[ignore = "flaky under load, br-xce6"]` if it fails again (in CI or a land), and later find the timing assumption that breaks under load (likely a fixed sleep or a short deadline around the hold/drain) and make it wait on the event instead.

Verify: passes 30 runs in a row while another `just check` runs alongside.

Model: sonnet. Low priority unless it reddens main.
