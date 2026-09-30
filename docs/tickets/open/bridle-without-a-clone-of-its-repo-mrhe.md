---
id: mrhe
title: Run bridle on a project without a local clone of the bridle repo
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: [where-the-knowledge-root-lives-and-migrating-to-it-ew97]
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> Long term -I should be able to run bridle without cloning the repo. Not urgent no big deal for
> now, but I should be able to install bridle and use it on a project standalone without the
> bridle repo locally.
>
> Obvisouly I wouldn't be building bridle itself there but that should also work fine.

Low priority, long term.

## Today (at 0a5c4d6)

A project needs a bridle clone on the same machine for:

- **The workflow layers**: each project's `.bridle/config.toml` names them by path, e.g.
  meta-notes' `workflow = "/Volumes/Data/work/bridle/bridle/workflow"` (the comment says it "can
  be a git url"; check whether that's built). Role prompts, rules and skills come from there.
- **The orchestrator and advisor**: started from `scripts/claude-orchestrator` and
  `scripts/claude-advisor` in the clone, reading `workflow/base/roles/*.md` relative to it
  (`bridle prime orchestrator` reads the current directory).
- **The binary**: `cargo install --path crates/bridle` from the clone.

## Shape (sketch)

- Install without a clone: `cargo install --git`, or a release binary.
- The base workflow either ships inside the binary (a default when a project names none) or is
  fetched from a git URL into `~/.bridle/` and updated by `bridle sync`.
- The launch scripts become commands, e.g. `bridle session orchestrator|advisor`, runnable from
  any directory.

## The human on the sketch (2026-09-29)

> Yep. Sounds like a good sketch. It won't block anything but it should be a destination state
> that we have. I'm fine with having the binary clone from GitHub in an initial step. It should
> match the workflow versions to itself probably with a tag. That is something that would drift
> though, unless the binary refreshed the local copy.
>
> OpenSpec works mostly like this. It vendors the skills into a project on init. Then you can
> update them later by choice so that your workflow doesn't change unexpectedly on you. Opt-in.

So (the advisor's reading):

- **A destination state**, not blocking anything.
- **Workflow version matches the binary**: the binary fetches the workflow from GitHub at the tag
  of its own version (a first-run or `init` step is fine).
- **Vendored on init, updated by choice** (like OpenSpec): `bridle init` copies the base
  workflow into the project; the project keeps that version until the human runs an update
  (e.g. `bridle workflow update`), so a new binary never silently changes a project's workflow.

**Conflicts with a recorded decision.** `docs/design/workflow-layers.md` ("Updates apply
automatically by default") records the human, 2026-09-28: no rev pinning; a project gets
whatever `workflow` currently has at its next `bridle sync`. That fits projects that point at
the local bridle clone (today's setup, where bridle's own workflow changes daily). Opt-in
vendoring fits standalone installs. Possibly both: auto for a path to a local clone, opt-in for
an installed binary. For the human to confirm when this is designed.

**Settled (the human, 2026-09-29):**

> Agree with this completely: The two decisions can coexist: projects that point at a local
> clone update automatically, and projects on an installed bridle update only when you choose.
