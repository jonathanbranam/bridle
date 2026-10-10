---
id: m5n2
title: "Onboard data-contracts: start the trial that was prepared on 2026-09-28"
kind: chore
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [63rv, rxe8, xfb3]
tasks: []
---

## The ask

The human, 2026-10-09 ~10:35 PM ET, via aide, verbatim: "if you have idle time, can you onboard
the data-contracts project to bridle? that shouldn't interrupt any other work since it doesn't
involve any changes to bridle."

## Where it stands (checked 2026-10-09 ~10:30 PM ET, advisor product-manager)

Most of the onboarding was done on 2026-09-28 and then stopped before the daemon was started:

- Survey: `docs/context/onboarding-data-contracts.md`. The human answered all 8 of its
  questions on 2026-09-28 (`docs/context/orchestrator-history.md`, "The data-contracts survey
  answers").
- data-contracts `main` (b923c7e): the superseded workflow tickets dropped, 992c landed, the old
  adopt branch and its worktree gone. Clean, on `main`, matches origin.
- Trial branch `bridle-adopt` (74f2aea + d73ce54, pushed, cut from b923c7e): `.bridle/config.toml`
  (prefix `dc`, packs python, integration `bridle-adopt`, `make check`, `uv sync --frozen`,
  max_workers 1, worker and manager roles as track-web), project rules, CLAUDE.md trimmed,
  OpenSpec CLI and skills retired. Its review note `.bridle/ADOPT-REVIEW.md` has the start-up
  steps and four open questions.
- Not done: the daemon was never started (no `~/.bridle/daemons/data-contracts.json`), no
  orchestrator token, no tasks.

## What's left (the orchestrator; no change to bridle's code)

1. Bring `bridle-adopt` up to date with today's bridle: compare its config and role settings with
   track-web's current `.bridle/config.toml` (12 days of bridle changes: launchd service, role
   prompt paths, sync output) and fix what's stale, on `bridle-adopt` only. Never `main`
   (rule `existing-projects`).
2. Start its daemon the way the other projects run now (launchd, as track-web), with the clone on
   `bridle-adopt`; the orchestrator token for data-contracts; the manager autostarts.
3. File the open library tickets as `dc-` tasks (vpaw, 5hwh, y3cr, 8tz3, ptwp, wdbz, wh6r),
   each linking its file; vpaw first.
4. Run vpaw with one worker as the first real task, landing on `bridle-adopt`.

## Open questions (ADOPT-REVIEW.md), proceeding on the recommended answers

The human can amend any of these in the morning; none blocks starting:

1. Library tickets become bridle tasks, vpaw first: **yes** (recommended).
2. `.claude/settings.json` written as `{}` by sync: likely moot, br-082f (sync no longer creates
   it without hooks) has landed; check.
3. acceptance-verifier: **keep** until `bridle-review` exists (recommended).
4. A 3.12-floor run in `make check`: **not yet** (recommended; ruff covers most).

## Order

After tonight's machine-setup work (the human, ~10:10 PM ET: machine setup has every slot
tonight). Steps 1-3 use no worker slot and can run tonight; step 4 (a worker) waits until the
machine-setup checks are through or a slot is free without delaying them.
