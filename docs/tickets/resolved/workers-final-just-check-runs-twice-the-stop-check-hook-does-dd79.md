---
id: br-dd79
title: "Workers' final just check runs twice: the stop-check hook doesn't count the worker's own green run"
kind: fix
created: 2026-10-11
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
resolved: 2026-10-11T22:41:10Z
---

## What happened

2026-10-11 ~11:28-11:40Z: workers wjstr (br-jstr) and wgd43 (br-gd43) each ran `just check` green,
committed, wrote their summary and done note, and ended the turn. The `bridle stop-check` Stop hook
then ran `just check` again at the new HEAD (hook processes 68733 and 22693, `just check` under each),
queued on the machine-wide test lock. wjstr sat 10+ minutes in the hook, and the daemon raised
`agent.stalled` for it (no events while a hook runs; manager-2's message m-10299 stayed `arrived`).

## Why it runs twice

The hook skips the check only when `.git/bridle-check-passed` records the current HEAD
(`crates/bridle/src/stop_check.rs`). The worker's own `just check` doesn't write that record, and
the workers' normal flow (check on the working tree, then `git commit`) changes HEAD anyway. So the
hook's check is a second full run on the same tree, every task. Each run is the ~30-40 min of
machine time the human counts per task (2026-10-09), and it holds the test lock that other workers
queue on.

## The ask

Count a worker's green `just check` on the tree it commits, so the hook doesn't repeat it. One
way: the recipe (or `bridle`) records the tree it passed on (`git write-tree` of the index/working
tree), and the hook accepts a HEAD whose tree matches. Also consider: the stall detector treats an
agent inside a Stop hook as stalled; a hook run could count as activity (or be named in the stall
note) so the orchestrator isn't woken for it.

Not critical: nothing breaks, it costs machine time. For the PdM to schedule.
