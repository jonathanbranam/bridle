---
id: br-ajp5
title: "A release daemon guesses bridle's repo from the project's origin: wrong workflow fetch and upgrades, and git prompts"
kind: bug
created: 2026-10-12
created_by: external:advisor/nuc-rename
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-r2nr, br-v7ug, br-eurm]
parent: br-kx5u
---

## The ask

The human, 2026-10-11 ~11:10 PM ET (to advisor nuc-rename), verbatim: "yes please file this bug;
Send to PdM - it should be part of the epic and land before I setup remus (wsl2) tomorrow."

What happened (the NUC rename, r2nr step 4, on vesta): `bridle serve --take-over` in the notes
clone stopped at `Username for 'https://github.com':`. notes's remote is SSH and `gh` is logged
in. Fixed for now by hand: `release_repo = "jonathanbranam/bridle"` under `[daemon]` in vesta's
`~/.bridle/config.toml`. After a restart `~/.bridle/workflow/` filled at once.

## Findings (advisor nuc-rename, 2026-10-11)

- vesta's machine config had `[daemon] self_upgrade = "release"` and no `release_repo`, as a new
  machine set up from the docs would. With no `release_repo` the daemon takes "bridle's repo" from
  the project's own `origin` (crates/bridle-daemon/src/lib.rs:456-490, `ensure_own_workflow`;
  docs/design/agent-host/daemon.md "Release upgrade" and "Workflow checkout").
- So every vesta daemon fetched its workflow checkout from
  `https://github.com/jonathanbranam/<project>.git` (workflow_checkout.rs:116, always https). A
  public project repo failed quietly (no `vX.Y.Z` tag); private notes made git ask for a username
  on the terminal (the clone in `workflow_checkout::ensure` doesn't set `GIT_TERMINAL_PROMPT=0`,
  unlike ci.rs). `~/.bridle/workflow/` had never been filled.
- The same fallback drives the release check, so vesta's daemons looked for bridle releases in
  each project's own repo and, it seems, never self-upgraded (not verified from logs).

## Wanted

- A release daemon never takes bridle's repo from the project's `origin`. Default to bridle's own
  GitHub repo (as `daemon init` already does for `workflow_url`, cli.md:74), or refuse loudly
  (doctor, start-up warning) when it can't tell.
- The workflow fetch (and any other git network call the daemon makes) never prompts:
  `GIT_TERMINAL_PROMPT=0`.
- `bridle doctor` flags a release daemon with no workflow checkout, or whose release repo isn't
  bridle's.
- The setup docs for a new machine (docs/context/add-a-machine.md, windows-wsl2-host.md) give a
  working `[daemon]` block, or need none once the default is right.

Needed before remus (v7ug) is set up, 2026-10-12.
