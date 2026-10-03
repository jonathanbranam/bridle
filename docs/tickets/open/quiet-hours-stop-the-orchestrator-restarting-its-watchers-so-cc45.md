---
id: cc45
title: Quiet hours stop the orchestrator restarting its watchers, so wakes pile up all night
kind: bug
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


Found 2026-10-03 by the orchestrator (incident log, 03:30 entry). The focus gate's quiet-hours text
says "no tool calls except the one the human asked for". The gate runs on every prompt submit,
including the background-task notification that a `wait-for-wake` exited. So the orchestrator
obeyed it and didn't restart its watchers. All three were down from about 23:30 to 09:07 ET.
Wakes queued: nothing was lost, but nobody saw two questions to the human, a red CI run on `main`
or a manager blocked on landing, until the human asked.

Wanted: restarting a watcher (`bridle orchestrator wait-for-wake`, `bridle agent wake`) and
reading what it printed are always allowed, in quiet hours too. Quiet hours limit what reaches the
human, not the watch loop. Either the gate text says so, or it doesn't apply to a turn that wasn't
started by the human. Acting on a wake (a red `main`, say) may still wait for the morning, by the
rule.
