---
id: br-kf7t
title: project start refuses a brand-new project (no owner), and take-over refuses it too (no bridle/state on origin)
kind: bug
created: 2026-10-14
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-sh8y]
related: [br-8uxe]
resolved: 2026-10-16T10:02:58Z
---

## The ask

The human, 2026-10-14 (to advisor wsl2), setting up the first home project on remus (ticket 8uxe): "all these bugs must be filed and fixed".

## What happened (advisor wsl2)

A brand-new project, created by the documented steps (`gh repo create`, `bridle init`, commit and push, `[projects]` entry, `bridle projects sync --apply`), can't be started by either command meant for it, bridle 0.7.2 on remus:

```
% bridle project start
error: remus-home has no owner recorded on bridle/state in /home/jbranam/work/remus-home-work/remus-home: run `bridle project take-over` here first
% bridle project take-over
error: take-over of remus-home refused: take-over refused: `bridle/state` not fetched from origin (local none, origin unknown): origin is unreachable or has no bridle/state branch
```

`start` sends the human to `take-over`; `take-over` refuses because `bridle/state` doesn't exist yet anywhere. Origin is reachable (the integration branch was just pushed); the branch simply has never been made.

## Why (from the code)

- Only `bridle serve` creates `bridle/state`: `StateBranch::open` makes it as an orphan on the project's first run, and with state push on it claims the owner and commits `owner.toml` before serving (`crates/bridle-daemon/src/lib.rs` ~561-572).
- `project start` checks for an owner before it installs or starts the daemon (`crates/bridle/src/project_start.rs:339`), so on a new project it never gets to the `serve` that would create one.
- `take-over` requires `bridle/state` fetched from origin (`TakeOverError::NotFetched`, `state_branch.rs` ~918); "no such branch on origin" is treated like "origin unreachable".

## What's wanted

`bridle project start` works on a brand-new project with no `bridle/state` anywhere: it creates the branch and records this machine as owner (what `serve` does on a first run), then installs and starts the daemon as usual. Keep refusing when `bridle/state` exists on origin with another owner, or when origin really can't be reached. `take-over` on a project with no `bridle/state` on origin should say "new project: run `bridle project start`" rather than "unreachable". Docs: the add-a-project and home-project (8uxe) steps say `start` is enough for a new project.

Workaround used meanwhile: run `bridle serve` once in the foreground in the clone (it creates the branch and claims the owner), stop it, then `bridle project start`.
