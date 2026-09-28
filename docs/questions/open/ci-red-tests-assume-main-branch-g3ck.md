---
id: g3ck
title: CI red since br-29f9: tests' repos start on master, spawn wants main
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [rxe8, f1ky]
---

## What happens

CI on `main` has failed on every push since c533cb0 (first failing run 17:49 UTC; 2b0b194
the last green). Reported by the human through the advisor (m-1075).

The `[branches]` merge (br-29f9, 0c84f5d) defaults the integration branch to `main`, so
spawn runs `git worktree add -b bridle/<name> <path> main`. The tests create repos with plain
`git init` (e.g. `crates/bridle/tests/cli_e2e.rs:40`, `crates/bridle-daemon/src/worktree.rs:310`,
`tasks.rs:938`, `state_branch.rs:721`), which on ubuntu-latest makes `master`:
`fatal: invalid reference: main`, about 21 tests across cli_e2e, governor, lifecycle, renew,
spawn_messaging and context_governor.

It passes locally, for workers and for the orchestrator's verification runs, because this
machine's global git config sets `init.defaultBranch=main`.

## The fix

1. Tests create repos with `git init -b main` (or set the branch explicitly), everywhere.
2. Tests run git with the user's global config isolated (e.g. `GIT_CONFIG_GLOBAL` pointing at
   an empty file, keeping `user.name`/`user.email` set explicitly), so local runs behave like
   CI and this can't hide again.
3. The daemon: when `[branches] integration` is unset and `main` doesn't exist, fail at
   startup (or config load) with a clear message naming the setting, rather than every spawn
   failing on `invalid reference`. A project on `master` sets it explicitly. (KISS: no
   guessing from HEAD; rxe8 wants strong, explicit branch rules.)

Done when CI is green on `main`.
