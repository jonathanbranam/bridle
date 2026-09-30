---
id: chvf
title: "Client machines stay current with bridle: workflow and daemon move together, by release"
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [mrhe, q7rx, wtyn]
---

## The ask


From the NUC's orchestrator (m-2869): a machine that runs only other projects' daemons (the NUC:
meta-notes) has no way to stay current with bridle.

- **Workflow**: agents read the base rules and roles live from the NUC's bridle checkout
  (`workflow` path in `~/.bridle/config.toml`). It only moves when someone runs `git pull`;
  on 2026-09-30 it was 16 commits behind until the NUC's orchestrator pulled it.
- **Binary**: `~/.cargo/bin/bridle` (0.3.0) only moves when the human runs `just install` (~11
  minutes) and then `bridle restart`. `[daemon] self_upgrade` doesn't help: it builds the
  daemon's own project, so the meta-notes daemon would build meta-notes.
- **They drift apart**: after the pull, the workflow names `bridle ticket`, which the NUC's
  0.3.0 binary lacks.

The human, verbatim (2026-09-30): "they're also working on publishing releases which could be a
potential fix here; I know bridle work is fast right now, but overall we need a solution while
bridle is changing rapidly so that client machines like nuc receive timely updates for workflow
changes with git pull and also timely updates to new bridle daemons."

## Proposal

Move the workflow and the binary together, to the same release, so a workflow never names a
command its binary lacks (mrhe's "workflow version matches the binary").

1. **Release often while bridle moves fast.** The release workflow (wtyn, br-ce9f) makes a
   release cheap. The orchestrator cuts a patch release whenever `main` has changes that client
   machines need (roles, rules, CLI or daemon), at most about daily. Later this could be
   automatic on green `main`.
2. **`[daemon] self_upgrade = "release"`** for client daemons: poll GitHub for the newest
   release, download the binary for this platform, check its SHA256, swap it in, and restart at
   a quiet point with q7rx's machinery (rollback and pre-flight included). No build on the
   client, so seconds instead of ~11 minutes.
3. **The daemon keeps its own workflow checkout, at the binary's tag.** Today one clone does
   both jobs on the NUC: the human pulls it to build, and projects' `workflow` paths point into
   it, so a pull changes every agent's workflow at once, before the new binary exists (the
   human, 2026-09-30: "we'd need to separate clones of bridle, right?"). Instead, when the daemon
   upgrades to `vX.Y.Z`, it fetches that tag into `~/.bridle/workflow/vX.Y.Z` (a shallow fetch
   or a `git worktree` from any clone; read-only by convention) and points the base layer there.
   Project overlays in `.bridle/` are untouched. With release binaries (step 2) the client
   doesn't build, so it needs no clone of its own; a clone the human keeps for development is
   never read by the daemon. Bridle's own machine keeps tracking `main` in its clone (the
   local-clone rule in mrhe's settled decision).

Until 1-3 land: when a bridle change matters to the NUC, dalek's orchestrator tells the NUC's
orchestrator to `git pull` its checkout, and to build in the background and ask for a restart
when the binary matters.

## Decided (2026-09-30)

The human approved the design, with step 3 as revised (the daemon's own workflow checkout at the
binary's tag) and the "at most about daily" release cadence. Steps 2 and 3 change the daemon's
upgrade and start-up path, so they land after the human's trip (not before Sat 2026-10-03).

## To decide

- Whether step 2 also replaces building on dalek (probably not: bridle's own daemon builds `main`).
