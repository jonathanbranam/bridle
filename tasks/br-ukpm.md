+++
id = "br-ukpm"
title = "A designer role: a background agent that reads a problem ticket, analyses the system and writes design options into the ticket"
kind = "feature"
state = "integrated"
created_at = "2026-10-06T01:21:56.084Z"
updated_at = "2026-10-09T07:13:56.626766Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/designfix"
commit = "5fa2d459d17e0c053e2ed15973892f07cf2aa50d"
summary = "Designer role: the base-workflow role 'designer' (commit 790b0273, carried on this branch) with rules that the role reuses by reference, plus the role's registration and docs. The branch now also merges main (d5cd2e04), which drops the stray crates/bridle/src/commands/mod.rs-e file and carries the j7r4 postmortem ticket exactly as on main. Human review is required before landing: the role and its rules are not to be used until the human approves them."
ticket = "ukpm"
+++

docs/tickets/open/a-designer-role-a-background-agent-that-reads-a-problem-tick-ukpm.md
Ticket (the ask with the human's words; read all of it):
Goal: a base-workflow role `designer` (workflow/base/roles/designer.md, plus any rules it needs under workflow/base/rules) for a daemon-spawned background agent. Input: one existing ticket describing a problem. Work: read it, analyse the current system (code, CLI, docs), then write options (several), a recommendation and the trade-offs, measured against stated design principles. Output: the proposal written INTO THE TICKET (the human: "I don't want a design document. I want a ticket."), committed; no design doc, nothing built; the designer's task ends when the proposal is in the ticket.
Principles, asserted strongly in the role: KISS (kiss.md), YAGNI (yagni.md), modularity, one name per action and no near-duplicate commands (ticket fne2), design read from the user's side first, plus a clearly marked place (a section or a separate rule file, e.g. workflow/base/rules/design-principles.md) where the human adds their own. Reuse the existing rules by reference, don't copy them.
Decide, and record in the role doc with the rejected alternative: one role with a focus (API/CLI surface vs internal architecture) or two roles. PM recommendation (YAGNI): ONE role, designer, with a "focus" section for each of the two sides, so the human can later split it if the prompts diverge; the human leans toward possibly two, so make the split a cheap rename.
State how designer differs from project-manager (plans and sizes work, doesn't analyse deeply or propose designs) and prototyper (builds a throwaway to learn; designer builds nothing). Use the prototyper role (ticket 6yb4) as the model for how a new base role is defined, registered and spawned; follow whatever it needed (role list, docs/README or roles index, the workflow's role table, tests that enumerate roles). Say in the done note how a project gets the new role (workflow sync) and that no per-project file changes are needed.
THE HUMAN REVIEWS the role and its rules BEFORE it is used: finish with a done note that lists the files and the key decisions for review, and the manager must not land it until the human approves (the orchestrator holds the landing; a worker finishing is not the work landing). Do not run the designer on any ticket; its first job (fne2) comes after the review.
Files likely: workflow/base/roles/designer.md, workflow/base/rules/*, wherever roles are listed/registered (grep prototyper), docs/design/workflow-layers.md or the roles doc, CHANGELOG.
Acceptance: just check passes (role-enumerating tests included); the role reads cleanly start to finish; ASCII only in docs the human edits.
Model: Sonnet. Out of scope: running it, the fne2 design itself, changing other roles.

## Thread

### note · agent:designer-role · 2026-10-06T02:54:25.042Z
done: designer role (one role, two focus sections) + design-principles rule, wiring, tests, docs. just check exit 0, 1262 tests passed; commit 9b730ed2. Summary is on the task. Awaiting the human's review; do not land yet.

### note · external:aide · 2026-10-09T00:45:23.232Z
From the human, via aide (2026-10-08 ~8:30 PM ET): "Add a clear design to w2hj before scheduling work. Also - fold the designer role into this as well. I approve that ticket. Get it moving." (Read by aide as: the designer role br-ukpm is approved to land, and the designer is one of the on-demand roles w2hj covers.)

### note · agent:designfix · 2026-10-09T06:36:04.966Z
done: NOT green; tip adf36526 unchanged, nothing new committed. Isolated runs: both previously failing tests pass alone (lifecycle_test::spawn_child_orphan_is_swept_on_stop 1/1, governor_test::working_agent_is_notified_then_stopped_when_its_turn_ends 1/1). Full just check (exit 100) failed on a third, unrelated test: bridle::cli_e2e sigint_shuts_down_cleanly_with_a_store_call_in_flight. Its panic is 'spawn failed: conflict: machine load is high (3.3 per core on 16 cores, threshold 2.5); spawns are held until it falls', so it is load-related, not a code failure. Summary: 70/1370 run, 69 passed, 1 failed. Suggest re-running the check once the other worker's load drops. I have not written a done summary or reported a checked commit.

### note · agent:designfix · 2026-10-09T07:12:58.752Z
done: green; tip d5cd2e04 (merge of main 0e72d190); just check exit 0, 1370 run, 1370 passed, 5 skipped. Not landed: the designer role needs the human's review first.

### note · agent:designfix · 2026-10-09T07:13:01.787Z
Checked tip d5cd2e04 (merge of main 0e72d190 into bridle/designfix): just check exit 0, 1370 run, 1370 passed, 5 skipped. Earlier failures on adf36526 were load-related timing flakes; both tests pass alone and the full run is green at a quieter load.

### note · agent:manager-2 · 2026-10-09T07:13:06.476Z
integrated: 5fa2d459d17e0c053e2ed15973892f07cf2aa50d (branch bridle/designfix)

### note · agent:manager-2 · 2026-10-09T07:13:56.626Z
cleanup: removed agent designfix, branch bridle/designfix
