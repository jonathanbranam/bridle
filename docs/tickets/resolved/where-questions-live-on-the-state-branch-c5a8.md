---
id: c5a8
title: Do questions get their own folder on the state branch, or live in the task thread?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [c7eb]
---

## The question

From `docs/design.md` §10.1 @ c192bfc, the state-branch layout:

```
questions/…               (or inline in the task thread — open, §15)
```

It was marked open but not listed in §15.

## Why it matters

Questions block tasks and answers are durable
([[docs/design/coordination#Messages|messages]]), so they need a home on the
state branch.

## Notes

- Design: [[docs/design/storage#The state branch|the state branch]].
- Depends on where task records live at all: [[task-records-on-a-state-branch-or-in-tree-c7eb|state branch or in-tree]].

## Resolution

A question lives inline in the thread of the task it blocks, not in a
separate `questions/…` folder. The daemon additionally indexes open
questions in SQLite so `bridle inbox` can show them without walking the
state branch. Recorded in [[docs/design/storage#The state branch|the state
branch]] and [[docs/design/coordination#Messages|messages]].
