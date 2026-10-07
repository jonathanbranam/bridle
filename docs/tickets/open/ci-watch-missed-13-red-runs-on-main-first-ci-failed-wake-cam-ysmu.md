---
id: ysmu
title: CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-ysmu]
---

## The ask

## What happened

On meta-notes-ui ([ci] github = true, daemon on 7405, project added 2026-10-05) CI on main failed from 97e0b13 (02:41 UTC) through df5f90d: 13 red pushes, 9 of them merges. The daemon recorded a single ci.completed event (df5f90d, 03:20:10) and the orchestrator's first ci_failed wake came then, about 40 minutes late. The manager kept merging on red: its allowlist denies gh, so it could not see CI and relied on the wake.

## The ask

- Find why the CI watcher missed runs (new project's watcher start? polling only the newest run? a run that finishes between polls dropped?). Fix the cause. Expected: one ci.completed event per run on main, and a wake on the first failure.
- Give the manager a bridle command that reads the latest CI result on main, so "never merge on red" does not need gh. Keep it small (a read of what the watcher already records, if possible).

Look at the CI watcher in crates/bridle-daemon and its design doc under docs/design/. Verify: just check, plus a test with a fake CI source returning several runs between polls (all recorded, first failure wakes).

Source: meta-notes-ui orchestrator, 2026-10-05.
