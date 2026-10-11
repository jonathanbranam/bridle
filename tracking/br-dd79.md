+++
id = "br-dd79"
title = "Workers' final just check runs twice: the stop-check hook doesn't count the worker's own green run"
kind = "fix"
state = "planned"
created_at = "2026-10-11T11:39:53.720Z"
updated_at = "2026-10-11T18:33:58.815948Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:wdd79",
]
summary = "The stop-check pass record in the git dir is now the tree id (HEAD^{tree}) instead of HEAD sha. bridle's just check ends with a new record-pass recipe that writes the working tree's id (temp index, git add -A, write-tree), so the worker's commit of the checked tree makes the hook skip its rerun; a changed tree reruns; the hook records the tree after its own pass. The hook writes <git-dir>/bridle-stop-check-running during its check and tick_stall_check skips agents with a marker under 30 min old. Other projects: the hook change applies automatically (hook runs once, records the tree); to skip it entirely they add a record-pass-style last step to their check. Docs: cli.md, CHANGELOG."
ticket = "dd79"
+++

Ticket: docs/tickets/open/workers-final-just-check-runs-twice-the-stop-check-hook-does-dd79.md (read it; human approved 2026-10-11, "approve as recommended"). First of its batch: the biggest throughput win (each duplicate run is ~30-40 min of machine time and holds the test lock).

Goal: a worker's green `just check` is counted on the tree it commits, so the `bridle stop-check` Stop hook does not run the check again. The hook (crates/bridle/src/stop_check.rs) skips only when `.git/bridle-check-passed` records the current HEAD; the worker's own run does not write that record and the commit changes HEAD. Build: record the tree the check passed on (git write-tree of the working tree/index, or equivalent) when `just check` goes green (the recipe or a bridle subcommand it calls; look at the justfile and how the record is written today), and have the hook accept a HEAD whose tree matches the recorded tree. Be careful about untracked/ignored files and a tree that changed after the green run: a changed tree must still run the check. Also: while an agent is inside a Stop hook, do not raise agent.stalled for it (count the hook as activity or name it in the note) so the orchestrator is not woken; find the stall detector in crates/bridle-daemon (supervisor.rs) and keep that change small.
Files: crates/bridle/src/stop_check.rs, justfile, crates/bridle-daemon/src/supervisor.rs (stall part), docs/design where the stop-check hook is described (grep stop-check docs/), CHANGELOG.
Migration: none for project files; the justfile recipe is bridle's own. If other projects' hooks call the same command, say how they pick up the change in the done note.
Acceptance: just check passes; tests: a recorded passing tree makes the hook skip after a commit of that tree; a different tree runs the check; a tree changed after the green run runs the check; a hook-run agent is not reported stalled. Model: Sonnet. Out of scope: changing what just check runs, the test lock.

## Thread

### note · external:advisor/product-manager · 2026-10-11T17:18:05.323Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve as recommended". Readied, normal; do it first of this batch (biggest throughput win).

### note · agent:wdd79 · 2026-10-11T18:04:30.395Z
done: worker's green just check now counts by tree id (record-pass step), hook skips rerun; no stall while hook runs; just check exit 0, 1530 tests passed, ran on 37bb67b2^1 content (main merged after, only ticket docs); commit 37bb67b2

### note · agent:manager-2 · 2026-10-11T18:33:58.815Z
main moved (docs only; br-e9wg landed). Merge main, run just check, message me the sha right away.
