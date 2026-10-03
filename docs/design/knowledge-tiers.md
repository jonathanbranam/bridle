# Project knowledge: the tiers

> **Status (checked 2026-10-03):** Built, not wired in: the parsers and local commands for every tier (`bridle goals`, `arch`, `spec`, `explore`, `trace`); no project has a `design/` tree yet, and no role or rule points agents at one · Planned: the approval column as a mechanism (no gates exist; see [[docs/design/gates|gates]])

Each project keeps its knowledge in four tiers of artifact, ordered from long
lifetime to short. They sit under a configurable root, `design/` by default:

| Tier | Where | Holds | Changes | Who approves a change |
|---|---|---|---|---|
| **Goals** | `design/goals/*.md` | long-term direction, some clear and some fuzzy | at planning sessions | **human** (agents may propose) |
| **Architecture** | `design/architecture/*.md` | project-wide design: principles, invariants, component boundaries, key decisions with alternatives | rarely, deliberately | **human, always** (locked rule) |
| **Specs** | `design/specs/<capability>.md` | detailed behaviour: requirements and scenarios | by tasks, continuously | manager, or human for `protected` requirements ([gates](docs/design/gates.md)) |
| **Explorations** | `design/explore/<task>/` | spike findings that may contradict everything above | per spike | nobody; they are not adopted by being written |

Tasks sit below all four, on the state branch ([[docs/design/storage#The state branch|state branch]]). Everything in the table
is in-tree and changes on code branches, so it is reviewed as a diff with the
work that motivated it.

Every element in every tier has a stable id (`g-`, `a-`, `r-`, `s-`), so the tiers
can link to each other ([[docs/design/traceability|traceability]]).
