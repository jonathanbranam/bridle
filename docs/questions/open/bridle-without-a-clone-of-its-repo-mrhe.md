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
