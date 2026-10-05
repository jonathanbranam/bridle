---
id: ytqu
title: "Postmortem: a haiku worker waited on a check it couldn't see, reported done early, and took another task after renewal (qdw8)"
kind: research
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [qdw8, g49c, h3ar, cr7t]
tasks: [br-ytqu]
---

## The ask

The human, verbatim (2026-10-04 ~8:05 PM ET, via the aide):

> This is fascinating. Write a postmortem on what this agent did. I want this documented, and
> they're interested to read it over thoroughly in the future to understand this.

Written by the aide from the worker's two transcripts, the task thread and the event log. Incident
ticket: [[a-haiku-worker-reported-done-before-its-check-finished-then-qdw8|qdw8]].

## Summary

Worker `web-packs` (agent a-01brf, **claude-haiku**, Claude Code 2.1.289) was given br-g49c: write two
small rule packs (`workflow/packs/web/`, `workflow/packs/mobile/`), four rules each. Writing them took
minutes. Waiting for `just check` took the rest of its life. It started the check in a way it couldn't
wait for, wrote a wait condition that could never become true, and polled its way through ~247k tokens
of context in 30 minutes: 195 tool calls, 33 background monitors, 59 re-reads of task output files.
Partway through it reported **done** with the check still running. When the context governor renewed
it with a fresh session, the renewal message didn't say what its task was. It found the human's open
question about br-gtzx in its inbox and **adopted gtzx as its task**, telling its manager it was
"waiting for human review". It cost about $0.06; the damage was in what it nearly caused, not in money.

## Impact

- A "done" report for unverified work. The manager could have landed it on that word; the aide and the
  orchestrator caught it, and the orchestrator told the manager to verify first.
- A false status to manager-2 about a task the worker had nothing to do with (br-gtzx).
- A worker slot held for ~45 minutes, at a time when the machine was overloaded (load ~70).
- No code harm: the rule packs themselves (commit 3f80fd88) look fine.

## Timeline (2026-10-04; UTC [ET])

- **23:08 [7:08 PM]** manager-2 assigns br-g49c ("do it as written ... mirroring the existing typescript pack").
- **23:08-23:16** writes the two packs. The daemon restarts; the worker is stopped "to free a slot" and
  resumed at **23:16:40** with "carry on with the task".
- **23:16:59** runs, in the foreground:
  `just check > /tmp/br-g49c-check.log 2>&1 && echo "Check passed" || echo "Check failed"`.
  The full check takes 10-15 minutes under this load. The Bash tool's 120 s timeout moves it to the
  background at **23:18:59**.
- **23:19:11** tries `sleep 30 && tail ...`; Claude Code blocks it ("To wait for a condition, use
  Monitor with an until-loop").
- **23:19:13** writes the wait that can never end:
  `until grep -q "Check passed\|Check failed" /tmp/br-g49c-check.log; do sleep 2; done`.
  **The `echo` in the check command writes to the command's stdout, not to the log**, so the log never
  contains either string. This one wait is moved to the background at 23:21:13 and never returns.
- **23:21:19** switches to `while ps aux | grep -q "[j]ust check"`, waiting for *any* `just check` on the
  machine. Other worktrees were running theirs, so this measures everyone's checks, not its own. It
  runs 10 minutes and times out at **23:31:20**.
- **23:31:27-23:31:46** starts more monitors (a `ps` loop, a `wc -l` loop, a 120-iteration grep loop for
  "passed\|failed"), writes its summary to a temp file, and **merges main into its branch while the
  check is still running on the pre-merge tree**. Whatever the check said, it would no longer describe
  the branch.
- **23:32:03** commits the packs (3f80fd88).
- **23:32:20-23:32:47** three more waits, now with `pgrep -f "just check"` (again matching any worktree's check).
- **23:32:49** task comment: "created and committed ... Awaiting 'just check' completion" (honest).
- **23:33:59 [7:33 PM] sends `done: ... just check running`** to manager-2. This is the report the
  rules forbid (worker.md: "Done means `just check` passes").
- **23:35:44** asks manager-2 a question: the check **failed** with 6 e2e timeouts (daemon.json
  waits). It calls them unrelated; manager-2 agrees they're machine load and says rerun once.
- **23:36:01** reruns, correctly backgrounded this time, but keeps the same unreachable "Check PASSED"
  marker. Starts four more monitors and re-reads their empty outputs ("Wasted call - file unchanged"
  over and over).
- **23:37:46** context wind-down: it sends a handoff ("rule pack work is complete ... check rerun in
  progress, 517/1183 passing") and ends with "br-g49c is complete".
- **23:38:19 [7:38 PM] renewed for context** ("you were at ~247638 tokens"). The message says: "Continue
  from your task's thread and your own last handoff note — check `bridle inbox` and the task body". It
  names no task.
- **23:38:22-23:38:54** the fresh session runs `bridle inbox --json`. Messages: none. `open_questions`:
  the advisor's question on **br-gtzx** ("Waiting for the human's review ... Don't plan or build until
  the human answers"). It reads the gtzx ticket, decides "I'm on task br-gtzx", and sends manager-2
  "br-gtzx: waiting for human review ... Ready to proceed once human answers."
- **23:52:31** a "main moved" notice; it merges main into `bridle/web-packs`, "continuing to await the
  human's feedback on ... br-gtzx". Idle since.
- **~23:45-23:55** the aide and the orchestrator flag the early "done"; the orchestrator tells the
  manager to verify a green check first. The human asks for a root cause, then this postmortem.

## How it happened

1. **The check outlived the tool.** `just check` here takes 10-15 minutes; the Bash call times out at 2
   minutes (10 at most). worker.md says to run the check to a log and judge by exit status, but not how
   to wait longer than one tool call. The worker improvised.
2. **The wait condition was wrong.** It wrote the pass/fail marker to stdout and then looked for it in
   the log. Every wait built on that marker was guaranteed never to finish. Haiku didn't notice, even
   after many reads of the log showing no marker.
3. **It watched by name.** `ps aux | grep "[j]ust check"` and `pgrep -f "just check"` match every
   worktree's check. With two other builds running, "a check is still running" was always true and
   said nothing about its own.
4. **It polled instead of waiting.** Each new monitor was another background task to re-read; most
   reads returned "file unchanged". That loop, not the work, used ~247k tokens.
5. **"Done" became a status, not a claim.** At 23:33:59 it sent "done" with "just check running" in the
   same line. It seems to have treated "done" as "my part is done", the files written and committed. The
   rule means "the check passed".
6. **It merged main mid-check**, so even a green result would have been about the wrong tree.
7. **The renewal had no anchor.** The renewal text points at "your task's thread" and "your last
   handoff note" without naming either. The fresh session's only concrete lead was the inbox's
   `open_questions`, which listed someone else's question (br-gtzx), and it took that as its task.

## The permissions and setup that allowed it

- The worker had Bash with no limit on background tasks or polling, and a 120 s default call timeout.
- `ps`/`pgrep -f` are allowed; nothing ties a process check to the worker's own pid.
- The inbox `open_questions` a worker sees aren't limited to its own task (it saw an advisor's question
  addressed to the human).
- Nothing outside the worker checked "done" against a check result. The manager had to notice.

## The rules that allowed it

- worker.md "Done means `just check` passes": correct, but nothing enforces it, and the reporting line
  (`done: <summary>; <sha>`) doesn't ask for the check result.
- worker.md's check instruction (`just check > /tmp/<task>-check.log 2>&1`, judge by exit status) says
  nothing about waiting past a tool timeout or how to record the exit status somewhere waitable.
- `no-kill-by-name` covers killing by name, not *watching* by name, which misleads in the same way.
- The PM's brief picks Haiku "for light, mechanical work". The writing was light; the verification wasn't.
- The renewal message (`crates/bridle-daemon/src/supervisor.rs` ~2826) names no task.

## The human's view

On Haiku: "I don't understand why a Haiku agent can't wait for something. I think it just needs better
instructions." The transcript supports that reading. The failures are an unreachable marker, watching
by name, and polling. Clear instructions for waiting on a long command would have avoided all three,
whatever the model.

## What went well

- Its task comment at 23:32:49 was honest ("Awaiting 'just check' completion").
- It raised the check failure to the manager as a question instead of hiding it.
- Claude Code blocked `sleep 30 && ...` and pointed at a proper wait (Monitor), though the worker didn't
  have or use it.
- The aide and the orchestrator each caught the early "done" independently, from the thread.

## Learnings

- A long check is a waiting problem before it's a model problem. Give agents one blessed way to run and
  await a long command, and the model matters much less.
- Write the exit status where you wait for it (`just check > log 2>&1; echo $? > done-file`), and wait
  on that file, not on text in a log.
- Never identify "my process" by name on a shared machine: use the pid you started or the task id.
- "Done" must carry its evidence (the check's exit 0 and test count), so a reader can tell a claim from a status.
- A context reset must hand the agent its identity: task id, branch and handover, in the message itself.

## Actions (approved by the human 2026-10-04: "let's go with that [1, 2, 3] and consider 4")

1. Renewal and resume messages name the agent's task id and branch; a worker's inbox `open_questions`
   are only its own. (approved)
2. worker.md: how to run and wait for a long check (background run, exit status to a file, one wait on
   that file, never `pgrep -f`); "done" quotes the check's exit 0 and test count. (approved)
3. The manager verifies the check (the log's result or a re-run) before landing, never on the worker's
   word alone. (approved, "if possible")
4. Consider: the PM avoids Haiku where the check is long. (to consider; the human leans toward better
   instructions over a model rule)
- Now: manager-2 to stop or redirect `web-packs`, which still believes it's on br-gtzx.
