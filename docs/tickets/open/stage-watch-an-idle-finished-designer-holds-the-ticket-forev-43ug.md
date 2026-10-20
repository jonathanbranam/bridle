---
id: br-43ug
title: "Stage watch: an idle, finished designer holds the ticket forever, so a redo after 'changes' never starts"
kind: bug
created: 2026-10-20
created_by: external:advisor/tj7q
theme: agents-and-cli
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-2ae8, br-ewk8, br-tj7q, br-t6ev, br-ewhy]
---

## The ask

Found in the human's hands-on try (br-ewk8, ticket br-2ae8). After `bridle ticket changes br-2ae8 specs ...` (2026-10-20 00:17Z, specs now unwritten, ticket unclaimed, row pending), no redo designer was started. `designer-2ae8-design` and `designer-2ae8-specs`, both finished long ago, sit `idle`.

Cause (`crates/bridle-daemon/src/stage_watch.rs`):
- A designer that has finished ends its turn and stays `idle`; nothing stops it. `AgentState::is_running` counts `Idle` as running.
- After a daemon restart (00:17Z upgrade) the watch re-adopts "a running `designer-<id>-<stage>` agent" by name prefix: it finds `designer-2ae8-design` (idle) and records it as the ticket's writer for stage `design`.
- `check_writer`: the design stage is done, and the agent "is running" (idle), so per br-t6ev it "still holds the ticket" and the watch spawns nothing. It never lets go while the idle agent exists.
- Without a restart the same holds for any finished writer kept in memory: an idle writer blocks a redo (or the next stage) indefinitely.

The fix (one clear way, pick the simplest that holds): a designer is done when its turn ends after a successful `stage-done` and no stage is left unblocked: the watch treats an idle writer as ended (only `Starting`/`Working` hold the ticket), and/or the daemon stops a designer once it ends its turn with nothing left to write. The re-adoption after a restart only takes a writer whose stage is still unwritten. Test: a ticket with an idle writer of a finished stage and a stage sent back by `changes` gets a new writer on the next tick.
