---
id: br-res3
title: "Code rename slice 3: no type is called Task; internal identifiers sweep (95t9)"
kind: arch-revision
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-92za]
related: []
parent: br-95t9
resolved: 2026-10-11T13:39:58Z
---

## The ask

Child of [[code-rename-no-type-is-called-task-types-wire-tasks-table-st-95t9|95t9]] (22ab step 9), split off by pm-1 on 2026-10-11 ~7:05 AM ET; made a child ticket by advisor product-manager so the order is a `blocked_by` link. pm-1's brief, as written:

Ticket: docs/tickets/open/code-rename-no-type-is-called-task-types-wire-tasks-table-st-95t9.md (read The ask and 22ab section 9 Code second pass). Slice 3 of 3, after slice 2 (wire types) is integrated. Goal: the rule of 22ab is that no type is called Task afterwards. Rename what is left: TaskManager, Task, TaskState, TaskKind and their functions, modules, variables and test names across every crate, to the ticket names slice 2 chose (the tracking row needs a name that is not task; the code design decides, so follow what slice 2 landed and record the name in docs/design/agent-host/ if not already). Mechanical but wide, so run it while little else is in flight and keep it one commit-able sweep that compiles at each step. Do not change wire names or behaviour (slice 2), and keep the wire aliases that accept old names. Docs: docs/design/agent-host/ identifiers where mentioned, CHANGELOG. Migration: none (no behaviour change); state it. Acceptance: just check passes; grep for the word Task as a type name in crates finds none (deliberate old-name serde aliases and comments excepted, listed in the done note); bridle ticket check reports no new problems. Model: Sonnet. Out of scope: wire, SQL, state folder, alias removal (step 10), bridle-ui.
