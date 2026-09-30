---
id: we2r
title: Push the state branch: task records exist only on the daemon's machine
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [project-machine-and-account-scope-9mxw, existing-projects]
---

## The ask

Found while the human and the advisor talked through running projects on the NUC (2026-09-29).
The human, verbatim:

> ok, so, that seems like a critical problem. Why istn' bridle/state pushed?

## Today (at 9023259)

- Each project's task records (bodies, threads, summaries, `queue.toml`, `edges.toml`,
  `claims.toml`) live on its `bridle/state` branch, committed by the daemon at most every 30 s
  (`crates/bridle-daemon/src/state_branch.rs`, `flush_now`).
- It's never pushed. `docs/design/storage.md` ("Bridle commits it"): "**Pushing on a configurable
  schedule is not yet built**; this build only commits locally." No reason beyond that, and no
  ticket until this one. `git ls-remote origin 'refs/heads/bridle/*'` is empty for bridle.
- The orchestrator's handover notes are worse off: SQLite only (`handovers`, SCHEMA_V17), "not on
  the state branch and not rebuilt". Before ct8m they were `docs/context/orchestrator-state.md`,
  committed and pushed.
- So a lost disk loses every project's task history and the latest handover; `bridle rebuild`
  reads the same local branch. And a project can't move machines (the NUC) by cloning.

## Shape

1. The daemon pushes `bridle/state` to `origin` after a flush that committed something
   (best-effort, debounced; a failed push is retried next time and shows in `bridle status`,
   not an incident per attempt). A fast-forward only, never forced: one daemon per project is
   the only writer.
2. Handover notes go on the state branch too (one file per note, or the newest only), so they're
   pushed with it.
3. `bridle rebuild` (or a new clone's first start) can fetch `origin/bridle/state` when there's no
   local branch.

## For the human

- **Existing projects** (`workflow/base/rules/existing-projects.md`): pushing adds a new branch,
  `bridle/state`, to meta-notes' and track-web's GitHub repos. It touches no existing branch.
  Is that approved for projects in trial, or bridle's own repo only at first?
- Task bodies go to GitHub wherever the repo lives; fine for private repos, worth a thought for
  public ones.
