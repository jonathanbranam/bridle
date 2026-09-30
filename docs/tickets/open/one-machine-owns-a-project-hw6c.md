---
id: hw6c
title: One machine owns a project; a clone that's only there to run bridle can't write to it
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [push-the-state-branch-we2r, bridle-without-a-local-clone-mrhe, project-machine-and-account-scope-9mxw]
---

## The ask

The human, 2026-09-30, setting up the NUC to run meta-notes:

> I already have bridle running on the NUC, but I told it not to push or do anything in the bridle
> repo - that seems like a funny thing to have to do. I don't want a stray comment causing a
> commit! to main or esp. to bridle/state. I think we need a different way to handle that.

## The problem

The NUC needs a bridle clone today only to build the binary and to read `workflow/` (mrhe).
Nothing stops that clone from being used as a project:

- a `bridle serve` there starts a second daemon for the project `bridle`. It commits to its own
  `bridle/state` and tries to push it. That's two writers: the push is refused as a non-fast-
  forward (we2r never forces), but the two histories have already diverged and someone has to
  sort them out;
- a Claude session started there (an orchestrator, an advisor, a stray `claude`) can commit to
  `main` and push it, racing the laptop.

Today the only guard is telling the agent not to. Instructions aren't a guard.

## Shape

1. **The state branch records its owner.** One file on `bridle/state` (e.g. `owner.toml`: host,
   daemon start time), written by the daemon that serves the project. `bridle serve` fetches
   `origin/bridle/state` first. If another host owns it, the daemon refuses to start and says
   which host and since when. Moving a project is explicit: `bridle serve --take-over` on the
   new machine, after the old daemon has stopped and pushed (the we2r push on shutdown). This
   also closes the ordering trap in the NUC runbook (fetch before the first flush).
2. **A clone can be marked tools-only on a machine**, in `~/.bridle/config.toml` (machine scope,
   9mxw), e.g. `[machine] tools_only = ["/home/<user>/src/bridle"]`, or anything not listed under
   the machine's projects. In a tools-only clone:
   - `bridle serve` refuses;
   - bridle installs a `pre-commit` and `pre-push` hook that refuse, with a message saying why
     (the human can still override with `--no-verify`, deliberately);
   - the orchestrator and advisor scripts refuse to start there, and start in the served
     project's workspace instead (the project-aware scripts, NUC task B).
3. **Long term, mrhe removes the need for the clone on a machine that only runs projects**
   (installed binary with the workflow vendored in). Until then, 1 and 2.

## Not in scope

- GitHub branch protection. It would block the daemon's own `bridle/state` pushes and the
  merger's pushes to `main` unless tokens differ per machine; heavier than 1 and 2 for the same
  result.
