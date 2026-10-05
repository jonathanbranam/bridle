---
id: qdw8
title: A haiku worker reported done before its check finished, then lost its task on renewal and adopted br-gtzx
kind: incident
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [g49c, h3ar]
tasks: [br-qdw8]
---

## The ask

The human, verbatim (2026-10-04 ~8:00 PM ET, via the aide): "I'm surprised the agent would do that. Do you
have a root cause? Let's analyze this to prevent it."

Worker `web-packs` (a-01brf, **haiku**), task br-g49c (two rule packs, mechanical). Transcripts:
`~/.claude/projects/-Volumes-Data-work-bridle-wt-web-packs/1f455950-...jsonl` (first session) and
`5810a6e2-...jsonl` (after renewal).

## What happened (UTC)

- 23:08 assigned; 23:16 resumed after a daemon restart; 23:16:59 ran `just check` in the
  **foreground** (the full suite takes 10-15 min under load; the Bash call can't wait that long).
- 23:32 committed, then lost track of its own check: polled it with `pgrep -f "just check"`, which
  also matches **other worktrees' checks** (load ~70, two builds running), so it couldn't tell whose
  check it was watching.
- **23:33:59 sent `done: ... just check running`**: done before the check finished, against
  worker.md ("Done means `just check` passes").
- 23:35 the check failed (6 e2e timeouts from load); the manager said to rerun; it reran.
- In 30 minutes: 195 tool calls, **33 background monitors, 59 re-reads of task output files** ("Wasted
  call - file unchanged"). Context hit ~247k on a 4-file task. The context wind-down at 23:37 made it
  send a handoff calling the work "complete", pending the check.
- 23:38 **renewed with a fresh context, it lost its task**: the renewal message ("Continue from your
  task's thread ...") names no task. `bridle inbox` showed open questions to the human, and it adopted
  **br-gtzx** (the human's seats review) as its own. It told manager-2 "br-gtzx: waiting for human
  review" and merged main. It's idle, believing that.

## Causes

1. **Haiku on a task with a long check.** The brief chose Haiku as "light, mechanical", but waiting
   on a 15-minute check under load wasn't light. Haiku broke the done rule and looped on polling.
2. **No safe way to wait on a long check.** worker.md says to run the check to a log and judge by exit
   status, but not how to wait past the Bash timeout. The worker improvised with `pgrep -f`, which
   breaks `no-kill-by-name`'s spirit (matches others' processes), and with dozens of monitors.
3. **Renewal doesn't carry the task id**, and the inbox shows unrelated open questions, so a renewed
   worker can pick up the wrong task.

## Fixes to weigh

- Renewal and resume messages name the agent's task id and branch; the open questions in a worker's inbox
  are only its own.
- worker.md: how to wait for a long check (one background `just check > log; echo $? > done-file`,
  then one wait on that file; never `pgrep -f`); "done" only with the check's exit 0 quoted.
- The manager verifies the check (log or re-run) before landing, never on the worker's word alone.
- PM: Haiku only when the check is short or the task needs no check wait; or the worker's check runs in
  the daemon (warm build), not in the agent's turn.
- Now: manager-2 should stop or redirect web-packs; it's idle and thinks it's on gtzx.
