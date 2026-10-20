---
id: br-sftn
title: "CI red on main after br-b2g8: check (ubuntu-latest) fails on 9e2b4858"
kind: fix
created: 2026-10-20
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-remg, br-ar4d, br-ck8a]
---

## The ask

Main 9e2b4858 (br-b2g8, a test-only commit adding a home-project test) fails CI check (ubuntu-latest): https://github.com/jonathanbranam/bridle/actions/runs/39512201248 . Find the failing test (the new b2g8 test is the first suspect: host or hostname dependent on the Linux runner; otherwise a known flake), and fix it. Acceptance: just check passes; CI green on the fix. Model: Sonnet.
