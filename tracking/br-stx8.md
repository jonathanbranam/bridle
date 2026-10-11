+++
id = "br-stx8"
title = "A task's state says what's really happening: held and built-awaiting-landing are states, not 'planned' with a note"
kind = "feature"
state = "planned"
created_at = "2026-10-06T02:04:20.046Z"
updated_at = "2026-10-11T17:19:28.236336Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
ticket = "stx8"
+++

Ticket: docs/tickets/open/a-task-s-state-says-what-s-really-happening-held-and-built-a-stx8.md. Read "The ask" and "Approved design (the human, 2026-10-11)": the design (pm-1 of 2026-10-06) is APPROVED AS WRITTEN with Q1-Q3 at the recommendations, and it supersedes the older brief that was here. The hold is lifted. Deferred, NOT in this task: the review state, the sent-back state, and settling names with v2va slice A's in_review (br-6ba3). Since 22ab the commands live under `bridle ticket`; `task` is the hidden alias. The code now says ticket (TicketManager etc.), so use the current names.

SLICE 1 (this task): daemon, wire, CLI, migration, docs. UI polish is a separate task (named in the thread).
States: existing unchanged (pending, open, planned, claimed, dropped, integrated, reopened); new exactly two, `held` and `built` (wire strings lowercase).
- planned: in the queue, claimable if deps are met and no open question.
- held: a person paused it. Not claimable, not spawnable; `bridle queue` shows it under its tier labelled held (Q1), never startable; an unmet dependency as planned is. Fields: held_by (principal), held_reason (required), held_until (free text, optional), held_at. Only from planned.
- built: finished on a branch, waiting to land or review. Requires branch and commit. Not claimable, not startable; does NOT resolve blocks edges (only integrated does). Left only by landing (done) or send-back. It is in the normal worker flow too (Q3): worker finishes -> built -> manager lands -> integrated; find where the worker's finish and the manager's landing set state today and move them through built without an extra manual step.
Commands (ticket forms, task aliases kept): `bridle ticket hold <id> --reason TEXT [--until TEXT]` planned -> held, thread note + `task.held`-style event (keep the event naming convention the code uses now, say which); `ticket unhold <id>` held -> planned; `ticket built <id> --branch NAME --commit SHA` claimed -> built (releases the lease) and planned -> built (parked work, by manager/orchestrator/human); `ticket done` built -> integrated (still allowed from claimed/planned as today); `ticket send-back <id> --reason TEXT` built -> planned keeping the branch (new command, Q2; reopen stays for dropped/integrated); drop allowed from held and built.
Transitions (anything else is 409): planned -> claimed|held|built|dropped; claimed -> planned|built|integrated|dropped; held -> planned|dropped; built -> integrated|planned|dropped; integrated/dropped -> reopened. Who: hold/unhold human, manager, PM, orchestrator; built the claiming worker or manager/orchestrator/human; done/send-back manager, orchestrator, human.
Migration (automatic at daemon start, idempotent, in storage.md): every ticket row in planned with branch and commit set and not integrated -> built. No guessing for held; list in the done note the candidates for the orchestrator to hold: br-vk3y, br-1ddd, br-91b3, br-6ba3, br-46fe, br-cf00, br-f610 (some may be gone). Also the queue, list and status output label held and built.
Files likely: crates/bridle-daemon (tickets.rs, store schema and migration, server.rs), crates/bridle-api/src/types.rs (additive wire change, accept old; daemon, CLI, TUI and gateway compile together), crates/bridle/src (commands, queue/status/list output), docs/design/storage.md (state machine), coordination.md (lifecycle), cli.md, manager and PM role docs where they describe planned/held/awaiting landing, CHANGELOG.
Acceptance: just check passes; tests: hold/unhold (not claimable while held, event and thread recorded), built reached by the worker finish and by the park command and left by landing, send-back, invalid transitions 409, queue labels, the migration (planned with branch becomes built; without stays; rerun no-op).
Model: Sonnet. If it grows past one branch, land items up to the migration and say what remains. Out of scope: TUI and gateway rendering (next task), review and sent-back states, kzw2.

## Thread

### note · external:aide · 2026-10-06T02:07:50.598Z
From the human, via aide (2026-10-05 ~10:30 PM ET), on br-stx8: "We're being way too fuzzy with these tickets and tasks we're handing off. I want to know exactly what these fields are going to be called before this ticket gets worked on. They just need to be designed better. We need designs. The worker agents that are doing these things are just making random guesses. ... I want to see on the ticket a proposed design before this goes anywhere else: Exactly what the names of the states are; Exactly how they work; A workflow of what the valid transitions are". Hold br-stx8 (no build) until a proposed design with those three is written into ticket stx8 and the human approves it.

### question · external:orchestrator · 2026-10-06T02:08:02.387Z
HELD by the human (2026-10-05 ~10:30 PM ET): no build until ticket stx8 has a proposed design (the exact state names, exactly how each works, and the valid transitions as a workflow) and the human approves it. The design is the designer role's job once br-ukpm lands; pm-1 may write it into the ticket for review instead. No worker on br-stx8.

### note · external:aide · 2026-10-06T02:08:14.481Z
From the human, via aide (2026-10-05 ~10:35 PM ET), on br-stx8: "Maybe just hold this up until we build a designer role agent first, and then pass it off to them. I'd be happy with that." So: br-stx8 stays held until the designer role (br-ukpm) lands; then hand stx8 to a designer to write the design (state names, how each works, valid transitions) for the human's approval before any build.

### note · agent:pm-1 · 2026-10-06T02:08:22.868Z
pm-1: PROPOSED DESIGN for stx8 (for the human's approval; no code until approved). I cannot edit files, so the orchestrator should paste this into ticket stx8 under "Proposed design".

1. STATE NAMES (task `state` field, wire string = lowercase)
Existing, unchanged: pending, open, planned, claimed, dropped, integrated, reopened.
New (exactly two): `held`, `built`.

2. HOW EACH WORKS
- planned: in the queue; a worker may claim it if deps are met and no open question. Nothing else.
- claimed: unchanged (a worker holds the lease). Returns to `planned` on release or lease expiry.
- held: a person has paused it. Not claimable, not spawnable, shown by `bridle queue` as "held" (never "startable"), does not block its dependents' edges any differently than planned (it is unmet until integrated). Fields on the task: `held_by` (principal), `held_reason` (text, required), `held_until` (free text, optional, e.g. "human reviews kzw2"), `held_at`. Set only from `planned` (a claimed task must be released first).
- built: work is finished on a branch and is waiting to land or for review. Fields: `branch` and `commit` (these already exist; both required to enter). Not claimable, not in the queue's startable list, shown as "built". Does NOT resolve `blocks` edges (only integrated does). Left only by landing (`task done`) or by being sent back.
- integrated, dropped, reopened: unchanged.

3. COMMANDS (exact)
- `bridle task hold <id> --reason TEXT [--until TEXT]`: planned -> held. Records a thread note and a `task.held` event.
- `bridle task unhold <id>`: held -> planned. Thread note, `task.unheld` event. (Not "release": that already means giving up a claim.)
- `bridle task built <id> --branch NAME --commit SHA`: claimed -> built (the worker's finish when the work is not landed yet; releases the lease), and planned -> built (for parked work, by the manager/orchestrator/human). Thread note, `task.built` event.
- `bridle task done <id> --commit SHA [--branch NAME]`: built -> integrated (today also allowed from claimed/planned; unchanged).
- `bridle task send-back <id> --reason TEXT`: built -> planned (review failed; keeps the branch field for the next worker). Thread note, `task.sent_back` event.
- `bridle task drop`, `reopen`: unchanged; drop is allowed from held and built too.

4. TRANSITIONS (the whole workflow; anything else is a 409)
pending -> open -> planned
planned -> claimed (claim) | held (hold) | built (built) | dropped
claimed -> planned (release/expiry) | built (built) | integrated (done) | dropped
held    -> planned (unhold) | dropped
built   -> integrated (done) | planned (send-back) | dropped
integrated/dropped -> reopened -> (as open)
Who may: hold/unhold: human, manager, PM, orchestrator. built: the claiming worker, or manager/orchestrator/human for parked work. done/send-back: manager, orchestrator, human (rule work-flow: only their merge accepts).

5. MIGRATION (automatic at daemon start, idempotent, in storage.md): every task in `planned` with `branch` and `commit` set and not integrated -> `built`. No guessing for held; the orchestrator holds br-vk3y, br-1ddd, br-91b3, br-6ba3, br-46fe, br-cf00, br-f610 with `task hold` afterwards. Rerun is a no-op.

6. OPEN QUESTIONS FOR THE HUMAN (my recommendations in brackets)
Q1: a held task still shows in `bridle queue` under its tier, labelled held? [yes, so the queue stays one list]
Q2: Is `send-back` a new command, or reuse `task reopen`? [new command; reopen is for dropped/integrated]
Q3: Should `built` be a state even for the normal worker flow (worker finishes -> built -> manager merges -> integrated)? [yes: that is the "waiting to land" state the manager reads]

Tell me when approved and I'll rewrite the br-stx8 brief with these exact names (and split UI/TUI polish into a second task).

### note · external:advisor/product-manager · 2026-10-09T11:04:39.266Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T17:54:59.632Z
advisor/product-manager (PdM), human via advisor (2026-10-10 ~2:20 PM ET), verbatim: "The task states work needs to be, I think, possibly superseded by, or it is definitely related to, the tasks as tickets work, and those two need to be in that epic and in the same theme. ... that needs to go with that, and that decision needs to be sequenced after that work. The first thing is to fix that: do all the ticket task work first because they're dependent and interrelated. That's a nice one. It'll be really nice to have, but it can come later." So: stx8 moves into epic everything-is-a-ticket (theme tickets-and-release), last, after all the ticket work. Blocked by br-72t9 (the last ticket step filed so far); the PdM re-points the block as later steps of 22ab's plan are filed. Still HELD for a proposed design and the human's approval; the design may find stx8 superseded by the ticket states of 22ab.

### note · external:advisor/product-manager · 2026-10-10T17:55:35.315Z
advisor/product-manager: correction, the human's words above were at ~1:55 PM ET, not 2:20.

### note · external:advisor/product-manager · 2026-10-11T04:12:48.340Z
advisor (product-manager): the human's input on pm-1's design (2026-10-11 ~1:45 AM ET) is in ticket stx8, section 'The human's input on the states': keep built; add a review state (manager checking) and a sent-back state (waiting for rework); settle names together with v2va slice A's in_review (br-6ba3).

### note · external:advisor/product-manager · 2026-10-11T04:14:01.010Z
Correction from advisor (product-manager): the human's input in my earlier comment was dated '~1:45 AM ET'; it was ~12:10 AM ET 2026-10-11 (the principle ~12:13 AM ET). Ticket stx8 fixed in aa70230b.

### answer · external:advisor/product-manager · 2026-10-11T17:19:02.607Z
human via advisor/product-manager, 2026-10-11 ~12:00 PM ET: "approve current design of stx8 as designed; we can deal with the review cycle questions later; let's land that first and refine". Hold lifted: pm-1's 2026-10-06 design is approved as written, Q1-Q3 at its recommendations; written into the ticket, section "Approved design" (7d023ff6). Commands under `bridle ticket` since 22ab step 7. Review and sent-back states deferred to a later refinement.

### note · agent:pm-1 · 2026-10-11T17:19:28.236Z
split off br-e4rv: stx8 part 2: held and built render with distinct labels in the TUI and gateway bindings
