---
id: br-qwd4
title: "project preflight: let the human name the clone's path on the new machine (it copies the old machine's absolute path)"
kind: feature
created: 2026-10-14
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kt25, br-kx5u]
resolved: 2026-10-15T20:52:33Z
---

## The ask

The human, 2026-10-14 (to advisor wsl2), moving data-contracts from dalek to remus: "I should be able to specify the destination path when running preflight; please file that as feature request". Then, readying it: "yes, ready and get the --path".

## What happened (advisor wsl2)

`bridle project preflight --to remus`, run in data-contracts' clone on dalek (bridle 0.7.2), offered to clone the project onto remus at dalek's own path:

```
clone the project onto remus (git clone git@github.com:jonathanbranam/data-contracts.git "/Volumes/Data/work/data-contracts-workspace/data-contracts")? [y/N]
FAIL  clone: project not cloned; on remus run: git clone ... "/Volumes/Data/work/data-contracts-workspace/data-contracts"
```

The path on remus should be `~/work/data-contracts-workspace/data-contracts`, the layout in docs/context/add-a-machine.md ("Workspace layout"). The human answered N and cloned by hand. Preflight then still reports "clone" as FAIL, and it skips the "git push" probe, which runs inside the clone.

## Why (from the code)

`crates/bridle/src/preflight_remote.rs` ~385 builds the remote path from the local one: under `$HOME` it becomes `$HOME/<rest>`, anything else is copied as an absolute path. dalek keeps its projects under `/Volumes/Data/work/`, outside `$HOME`, so the path is copied unchanged.

## What's wanted

- Must have: a `--path` option naming the clone's path on NEW when running preflight, for example `bridle project preflight --to remus --path ~/work/data-contracts-workspace/data-contracts`. The clone prompt, the clone check and the git push probe all use it.
- Nice to have: with no `--path`, a better default than copying OLD's absolute path: the documented `~/work/<P>-workspace/<P>` when OLD's path isn't under `$HOME`, or a clone that already exists on NEW. Up to the planner.
- docs/context/move-a-project.md step 1 and the CLI doc mention the option.
