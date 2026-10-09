---
id: qpr7
title: bridle uses its own specs, starting with how every command finds its project
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [3397]
tasks: [br-qpr7]
closed: 2026-10-09T23:11:07Z
---

## The ask

The human, verbatim (2026-10-04, via the aide), accepting br-3397 as built and asking for the
stronger enforcement as a follow-up:

> Agree with 2 - we need to start using specs first bridle

(Option 2 was: land 3397 now, and file a follow-up for a test that really runs each command from a
project folder, plus the refuse-duplicate-session idea. "first bridle" reads as "for bridle".)

bridle has the spec tooling built (`docs/design/specs.md`, `specs-to-tests.md`, `spec-flow.md`:
`bridle spec check|id|export|coverage`) but no project uses it, bridle included: there's no
`design/specs/` here. This ticket is bridle adopting its own specs, following
`docs/design/spec-flow.md`, "Adopting, in order".

## First capability: how a command finds its project

From [[bridle-session-ignores-the-folder-it-runs-in-and-defaults-to-3397|3397]] (the human: "every
command should include this behavior. There should be a spec enforcing that behavior for all
commands"). 3397 built the shared resolver (`crates/bridle/src/project.rs`) and a table test
(`SCOPES`: each top-level command's scope, failing when a new command is missing). It doesn't run
any command. The spec should:

- state the requirement: every project-scoped command picks its project from `--project`, then
  `$BRIDLE_PROJECT`, then the workspace containing the cwd, and refuses when none resolves
  (never a hard-coded `bridle`);
- have an **executable** scenario that runs every project-scoped command (subcommands too, not just
  top-level) from a temp workspace for project X with no `--project`, and checks it resolved X (or
  refused when outside any workspace). A command wired wrong fails, not just one missing from the table;
- be linked to the tests through `bridle spec coverage`, as the spec flow describes.

## Then

Which further capabilities bridle specs first, and whether the spec flow becomes a rule for bridle's
own tasks (tasks edit `design/specs/` in place). That's to decide while doing this one, not up front.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
