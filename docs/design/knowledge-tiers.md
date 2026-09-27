# Project knowledge: the tiers

Each project keeps its knowledge in four tiers of artifact, ordered from long
lifetime to short. They sit under a configurable root, `design/` by default:

| Tier | Where | Holds | Changes | Who approves a change |
|---|---|---|---|---|
| **Goals** | `design/goals.md` | long-term direction, some clear and some fuzzy | at planning sessions | **human** (agents may propose) |
| **Architecture** | `design/architecture/*.md` | project-wide design: principles, invariants, component boundaries, key decisions with alternatives | rarely, deliberately | **human, always** (locked rule) |
| **Specs** | `design/specs/<capability>.md` | detailed behaviour: requirements and scenarios | by tasks, continuously | driver, or human for `protected` requirements ([gates](docs/design/gates.md)) |
| **Explorations** | `design/explore/<task>/` | spike findings that may contradict everything above | per spike | nobody; they are not adopted by being written |

Tasks sit below all four, on the state branch ([[docs/design/storage#The state branch|state branch]]). Everything in the table
is in-tree and changes on code branches, so it is reviewed as a diff with the
work that motivated it.

Every element in every tier has a stable id (`g-`, `a-`, `r-`, `s-`), so the tiers
can link to each other ([[docs/design/traceability|traceability]]).
