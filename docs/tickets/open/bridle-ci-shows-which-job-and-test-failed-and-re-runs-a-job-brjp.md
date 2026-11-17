---
id: br-brjp
title: bridle ci shows which job and test failed, and re-runs a job GitHub cancelled before any step ran
kind: feature
created: 2026-11-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-8xas]
---

## The ask

When CI on the integration branch goes red, the manager can tell from bridle alone what failed and whether it is infra, without `gh` (denied to it) and without the orchestrator:

1. `bridle ci` (and the `ci_failed` wake text) names each failed job, and for a test failure the failing test names and the first panic/assert line.
2. A job that GitHub cancelled before any step ran (no hosted runner) is infra, not a code failure: the daemon re-runs the failed jobs once by itself and says so; only a second such cancel wakes anyone.

## Why (2026-11-17, bridle on dalek)

- Twice today `check (macos-latest)` was cancelled after ~15m with zero steps run (runs 44114639704 and 44151755367); ubuntu passed. Both times main read red, manager-2 held every landing, and could only ask the orchestrator to read the run and re-run it (`gh` is denied to managers). Each round trip cost 30-60 minutes, more during focus hours when the orchestrator defers.
- Once (run 44141420535) the failure was real (br-ggnp's test, fixed by br-hnef); manager-2 also could not see which test, so it could not send a worker until the orchestrator read the log.

## Notes

- Related: br-8xas (bridle ci ignores a passing re-run); this builds on the same poller, and the auto re-run needs that fix to unblock landing.
- Rejected: giving managers `gh`. Wider access than needed (rule-of-least-access, see gz4p); the daemon already talks to GitHub for CI.

## Verify

A test with a fake GitHub API: a run whose job was cancelled with no steps is re-run once and its passing re-run turns `bridle ci` green; a run with a failing test reports the job and test name.

Model: Sonnet.
