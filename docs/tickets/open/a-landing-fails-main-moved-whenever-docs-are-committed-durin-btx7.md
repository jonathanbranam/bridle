---
id: btx7
title: A landing fails 'main moved' whenever docs are committed during its check
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-btx7]
---

## The ask

Found by the orchestrator, 2026-10-05 ~01:00Z. Not yet the human's decision; a bug in landing.

## What happened

br-bnhn's `bridle task land` failed three times with "main moved, retry" (2026-10-05 00:28Z to
01:15Z). The land check takes ~10 min at load ~40, and during each run docs-only commits landed on
local `main`: tickets and incidents from the orchestrator and aide, and the doc-review watcher's
automatic "review: human comments on ..." commits. The orchestrator asked aide, advisors and pm-1 to
hold commits, but the watcher's commits aren't held by anyone. manager-2 got it through by having
the worker merge the docs-only delta (no re-check) and fast-forwarding at once.

Impact: ~45 minutes of a landing and a worker slot; every role told to stop committing.

## Fix to weigh

`task land` doesn't fail when `main` moved during the check only by commits whose paths can't affect
the check: it merges them in and lands without re-running it. Which paths are safe is a project
setting (e.g. `[integration] check_skip_paths = ["docs/**"]`; not `design/specs/`, which the check
reads). Anything else still fails "main moved" as today.
