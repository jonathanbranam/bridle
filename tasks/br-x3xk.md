+++
id = "br-x3xk"
title = "is_merged calls a reused branch landed: an old Branch: trailer hides newer unlanded commits"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T13:39:51.920Z"
updated_at = "2026-10-04T16:06:09.993102Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/merged-trailer"
commit = "2c8199dc6ff0454cef8f7b0ffc9a61f9664751fd"
summary = "worktree::is_merged: a 'Branch:' trailer on HEAD now counts only if no commit in HEAD..branch has a committer date after the landing commit (reused branches stay unmerged). No change to landing; both callers (status/rm gate) get the fix. Test extended for landed-then-reused; docs (operating-model) and CHANGELOG updated. Caveat: date-based, second resolution."
ticket = "x3xk"
+++

Bug. Ticket: docs/tickets/open/is-merged-calls-a-reused-branch-landed-an-old-branch-trailer-x3xk.md (read it; cause and examples are there). worktree::is_merged (crates/bridle-daemon/src/worktree.rs ~477) counts a branch as merged when any commit on HEAD carries a 'Branch: <branch>' trailer, so a reused branch with newer unlanded commits is called landed; bridle status then suggests 'bridle rm --delete-branch' and following it with --force would delete unlanded work (seen with bridle/self-upgrade, br-88d4). Fix: a trailer counts only if the branch tip adds nothing after what that landing took (e.g. compare the branch tip to the landing, or git cherry shows nothing unlanded), or have the landing record the tip; pick the smaller one that works for already-landed history. Check the other caller, supervisor.rs ~2881, for what it gates. Tests: fully landed branch => merged; landed then more commits => not merged; never-landed => not merged. Docs: the design doc that describes 'merged' in bridle status, CHANGELOG. Both this and br-4f8y touch worktree.rs: land this first, the other rebases. Acceptance: just check passes. Model: Sonnet. Migration: none. Do not delete or touch the bridle/self-upgrade branch or worktree.

## Thread

### note · agent:merged-trailer · 2026-10-04T15:33:52.260Z
done: is_merged ignores an old Branch: trailer when the branch has commits newer than the landing; just check green (1151 passed); c62904cb

### note · agent:merged-trailer · 2026-10-04T15:48:57.629Z
main merged; just check green (1154 passed; two load-timing tests failed once, passed on re-run); tip 997e2dae

### note · agent:manager-2 · 2026-10-04T16:02:21.986Z
integrated: 2c8199dc6ff0454cef8f7b0ffc9a61f9664751fd (branch bridle/merged-trailer)

### note · agent:manager-2 · 2026-10-04T16:06:09.993Z
cleanup: removed agent merged-trailer, branch bridle/merged-trailer
