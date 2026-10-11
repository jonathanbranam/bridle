---
id: br-95t9
title: "Code rename: no type is called Task (types, wire, tasks table, state branch folder), compatible for one release"
kind: arch-revision
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-22ab]
parent: br-22ab
---

## The ask

Parent: br-22ab (step 9 of its plan, approved by the human 2026-10-09 ~4:40 PM ET: "approve; looks great"; "run at a quiet time: it touches every crate"). Its gate, "after step 7", cleared on 2026-10-11 ~7:04 AM ET when br-aejj landed (step 7: br-q8cs, br-sehn, br-aejj). The design is 22ab section 9, "Code (second pass)"; read it and section 12 (what doesn't change).

The human, 2026-10-11 ~12:30 AM ET, to advisor product-manager: "I'd love the everything is a ticket epic to land or get close by morning ... I don't want epics to drag on for too long."

Scope (22ab section 9, verbatim where quoted):

- `TaskManager`, `Task`, `TaskState`, `TaskKind` (wire types in `bridle-api/src/types.rs`), the `tasks` table, and the state branch's `tasks/` folder.
- "The tracking row needs a name in code that isn't 'task' ... Leave that to the code design, within one rule: no type is called `Task` afterwards."
- Wire changes go to all clients and the daemon together, and stay compatible for one release (accept old and new names): the lesson of incident 2ax5, where a breaking wire change broke cross-project messaging. bridle-ui reads the gateway: the old gateway routes from br-aejj stay.
- Storage: a schema migration for the table rename, and the state branch folder moved with a reader that accepts the old folder for one release (other projects' state branches: no bulk rewrite; rule existing-projects).

Out of scope: removing the `bridle task` alias and old-name readers (step 10, one release later); bridle-ui (step 8, its own project); states (stx8); parts and gates (95mu).

Size L, every crate. Split into slices as child tickets (`parent: br-<this>`, ordered with `blocked_by`) if it is too big for one worker, and run them while little else is in flight.

Acceptance: `just check`; an old client and the new daemon still talk (wire compatibility test); a state branch with the old `tasks/` folder still loads; `bridle ticket check` reports no new problems.
