+++
id = "br-upxq"
title = "Stages: a designer's stage must be on main before the human is asked to review it"
kind = "bug"
state = "claimed"
created_at = "2026-10-18T15:08:02.155Z"
updated_at = "2026-10-19T00:54:58.228145Z"
created_by = "external:advisor/tj7q"
watchers = [
    "external:advisor/tj7q",
    "agent:wupxq",
]
priority = "urgent"
priority_at = "2026-10-18T15:08:07.318951Z"
summary = """stage-done from a designer's bridle/* worktree now commits the marks and calls the new POST /v1/tickets/{id}/stage-land (types StageLandRequest/Result, client stage_land). The daemon (stage_land.rs, under the landing lock) refuses a branch touching anything but the ticket file and its stage folder (path named), uncommitted edits to those paths on main, or a conflict (aborted, main left clean); else merges and adds the "stage X written ..." thread note naming the main commit. No row state change, no done_task. stage_watch: prompt drops the manager step; a writer that ended is alerted unless the stage is on main (written_in_worktree removed), one note + one message to the product manager. stage-done outside a bridle/* branch (filer's own checkout) keeps the old in-place behaviour. Tests: 4 git tests in stage_land.rs (lands and readable on main, outside path refused, conflict refused, dirty main refused). Not tested: the end-of-writer alert (unchanged code path, no existing harness) and an end-to-end CLI/daemon run."""
ticket = "upxq"
+++

docs/tickets/open/stages-a-designer-s-stage-must-be-on-main-before-the-human-i-upxq.md

## Thread

### note · external:advisor/tj7q · 2026-10-18T15:08:07.318Z
priority: normal -> urgent

### note · agent:pm-1 · 2026-10-18T15:08:47.716Z
Brief (pm-1). Model: Sonnet. Urgent bug in the Stages flow (br-ewhy stage watch, br-tj7q).
Read: docs/design/stages.md, the stage-done code and the stage watch in crates/bridle-daemon (grep stage-done, designer), the designer spawn prompt.
Recommended fix (simplest; rejected: a manager remembering to land branches): stage-done writes the stage file on main the same way ticket files are committed to main (ticket commands already do this), so no designer branch needs landing. If that is not feasible in the code, the stage watch lands (fast-forward or merge) the designer's branch to main right after the stage-done commit. Pick one, state which in the stages.md edit.
Must hold: every stage (not only review: human) is on main before any "waiting for signature" note or message to an advisor/human is sent, and the note names the main commit. The designer prompt names its manager (look it up, e.g. from the spawning agent) so `bridle send` works.
Test: a designer stage with review: human is readable on main (ticket show) when the signature request goes out. Docs: stages.md, CHANGELOG. Acceptance: just check.
Out of scope: br-2ae8's own proposal (already put on main by hand), other stage-flow redesign. Touches the same area as the Stages tickets: run alone with them.

### note · external:advisor/tj7q · 2026-10-18T15:09:35.481Z
From the human, via advisor tj7q: "stop the worker; should be 10min hold". Don't plan br-upxq or start a worker on it: the human is choosing its design with me now. I'll write the decision into the ticket and tell you when it can go.

### note · external:advisor/tj7q · 2026-10-18T15:10:20.658Z
From the human, via advisor tj7q, correcting my last message: "the 10 min hold exists and should be followed as normal". So no special hold on br-upxq: plan it as usual and let the normal settle period run. I'll add the human's design decision to the ticket as soon as they choose.

### note · external:advisor/tj7q · 2026-10-18T15:15:03.866Z
From the human, via advisor tj7q: "agree, yes, approved and ready to work". The human's design decision is now in br-upxq (## Decision, commit 0dc348ef): stage-done asks the daemon to verify the designer's commit touches only the ticket and its stage folder, the daemon merges it to main, then records the stage; failures go back to the designer, then the product manager. Plan it as usual after the normal settle.

### note · agent:pm-1 · 2026-10-18T15:15:08.201Z
pm-1: the human's ## Decision in the ticket file (commit 0dc348ef) supersedes the "Recommended fix" in my brief above: follow the Decision (stage-done asks the daemon to verify the commit touches only the ticket and its stage folder, the daemon merges to main, then records the stage; failures go to the designer, then the product manager). The rest of my brief stands (names the manager in the prompt, test, docs, CHANGELOG, just check, Sonnet).

### note · external:orchestrator · 2026-10-18T21:16:45.156Z
From orchestrator: br-upxq is urgent and takes the next free worker slot once br-fyww has landed (same files). The human, via advisor tj7q, on the Stages try (br-ewk8): "It's ok just keep that moving". Tell me the moment fyww lands: I reopen br-2ae8 (to pending) then.

### note · agent:wupxq · 2026-10-19T00:54:58.228Z
br-upxq: built, check green (exit 0, 1885 tests) on 853ecff2; main merged after that (HEAD e63e6b6b) and the re-check is stuck behind test lock pid 86314 (integration, 1h13m): is that landing hung? Tell me to wait, or land on the green sha.
