+++
id = "br-res3"
title = "Code rename slice 3: no type is called Task; internal identifiers sweep (95t9)"
kind = "arch-revision"
state = "pending"
created_at = "2026-10-11T11:05:22.388Z"
updated_at = "2026-10-11T11:05:22.389457Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
parent = "br-95t9"
+++

Ticket: docs/tickets/open/code-rename-no-type-is-called-task-types-wire-tasks-table-st-95t9.md (read The ask and 22ab section 9 Code second pass). Slice 3 of 3, after slice 2 (wire types) is integrated. Goal: the rule of 22ab is that no type is called Task afterwards. Rename what is left: TaskManager, Task, TaskState, TaskKind and their functions, modules, variables and test names across every crate, to the ticket names slice 2 chose (the tracking row needs a name that is not task; the code design decides, so follow what slice 2 landed and record the name in docs/design/agent-host/ if not already). Mechanical but wide, so run it while little else is in flight and keep it one commit-able sweep that compiles at each step. Do not change wire names or behaviour (slice 2), and keep the wire aliases that accept old names. Docs: docs/design/agent-host/ identifiers where mentioned, CHANGELOG. Migration: none (no behaviour change); state it. Acceptance: just check passes; grep for the word Task as a type name in crates finds none (deliberate old-name serde aliases and comments excepted, listed in the done note); bridle ticket check reports no new problems. Model: Sonnet. Out of scope: wire, SQL, state folder, alias removal (step 10), bridle-ui.
