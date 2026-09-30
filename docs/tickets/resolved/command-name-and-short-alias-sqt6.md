---
id: sqt6
title: Command name and short alias
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [geem]
closed: 2026-09-30T05:12:44Z
---

## The question

From `docs/design.md` §15 @ c192bfc, item 7:

> **Command name.** `bridle` in full. `br` belongs to beads_rust, which otters
> still uses. The short alias is still undecided.

## Why it matters

Agents type it on every call, and a clash with `br` breaks otters.

## Notes

## Resolution

No short alias. The human, 2026-09-28:

> sqt6 - no short alias; I want to finish the first rounds of getting it working, then I
> actually was planning to come up with a new, unique name for the entire project.

and again, the same day:

> I don't want to stick with bridle long-term; we can come up with a short name after
> picking a new overall name.

The command name, and any short alias with it, is now part of
[[a-new-name-for-the-project-geem|a new name for the project]]; the names to avoid are in
[[docs/context/agent-harness-name-catalogue|the harness name catalogue]].
