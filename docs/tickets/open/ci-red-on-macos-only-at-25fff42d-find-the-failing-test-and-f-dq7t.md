---
id: br-dq7t
title: "CI red on macos only at 25fff42d: find the failing test and fix"
kind: fix
created: 2026-10-22
created_by: agent:manager-2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

CI run https://github.com/jonathanbranam/bridle/actions/runs/39713596548 failed on check (macos-latest) only (ubuntu passed) at 25fff42d (br-xvb2 fix landing). Read the log (gh run view 39713596548 --log-failed), name the failing test, and decide: real macOS bug, or a load flake (then fix the cause, per the flaky-test rule, not a longer timeout unless plain slowness). Acceptance: just check passes; say which test and why. Model: Sonnet.
