---
id: 93u2
title: Integration branch per project, or merge straight to main?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

From `docs/design.md` §15 @ c192bfc, item 3:

> **Integration branch per project, or merge straight to main?** With
> after-merge acceptance, rejected work on main needs a revert. An integration
> branch that the human fast-forwards to main on acceptance is safer and adds
> a step.

## Why it matters

It decides what "integrated" means in the
[[docs/design/roles-and-lifecycle#Task lifecycle|task lifecycle]] and how
rejected work is undone.

## Notes

- [[docs/design/gates|Gates]]: `[gates.accept] when = "after-merge"` is the
  default, with `before-merge` suggested for harness.
- [[docs/proposal/build-order|Build order]] P5: the integrator (merge-tree probes, merging
  workers' branches) comes after v1.

## Resolution

Decided to merge worker branches straight to main with `--no-ff`, as described
in `docs/design/agent-host/operating-model.md`, "Merging completed work"
section (lines 60–74). The merger in main checks that the branch contains
main, validates the diff, and merges with `git merge --no-ff bridle/<agent>`.
An integration branch and integrator role remain open in build-order P5.

Resolved 2026-09-28.
