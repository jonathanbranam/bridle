+++
id = "br-stx8"
title = "A task's state says what's really happening: held and built-awaiting-landing are states, not 'planned' with a note"
kind = "feature"
state = "planned"
created_at = "2026-10-06T02:04:20.046Z"
updated_at = "2026-10-06T02:08:22.868766Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "stx8"
+++

docs/tickets/open/a-task-s-state-says-what-s-really-happening-held-and-built-a-stx8.md
Ticket (the ask with the human's words and the aide's breakdown of the 24 planned tasks; read all of it). The human is upset: the UI shows 24 "planned" tasks, only 2 being worked, and nothing tells them apart.
Goal: a task's state says what is really happening, visible at a glance in the CLI and the UI.
Build:
1. A `held` state: a held task is not startable and not in the way of the queue; it records who holds it, why, and until what (free text is fine for "until", e.g. "human reviews kzw2"). Commands set and clear it (propose `bridle task hold <id> --reason ... [--until ...]` and `bridle task unhold <id>`, which returns it to `planned`; avoid colliding with `task release`, which releases a claim; follow the CLI's naming conventions and say why). Hold and unhold are recorded in the thread and as events. A held task cannot be claimed or spawned; `bridle queue` shows it as held, not startable.
2. A state for "built on a branch, waiting to land or for the human's review" (propose `built`, or the name the state machine's existing vocabulary suggests): how a task gets there (find how work is parked today: tasks with a branch and commit that are not integrated, e.g. br-6b8a, br-2718, br-2672, br-8b98, br-79c3, br-88d4, br-751e; use the existing worker-done/landing flow so no extra manual step is needed), and that landing it moves it to integrated as today. Wherever the manager or orchestrator reads "waiting to land", it is now a state, not a note.
3. `planned` then means only "in the queue, a worker can start it".
4. MIGRATION (required, automatic, at daemon start, idempotent): tasks in `planned` that have a branch/commit recorded and are not integrated move to the built-awaiting-landing state. Held cannot be detected reliably from text, so the migration does NOT guess; instead list in the done note the held candidates from the ticket (br-vk3y, br-1ddd, br-91b3, br-6ba3, br-46fe, br-cf00, br-f610) so the orchestrator can hold them with the new command. Document in docs/design/storage.md.
Show the states in the UI: state is already displayed as text in the task lists; make sure the new states render with distinct labels/styles wherever states render (crates/bridle-gateway bindings/*.ts and the UI source it serves; the TUI too), and that `bridle task list`/`queue`/`status` group or label them. Update docs/design/storage.md (state machine), coordination.md (task lifecycle), cli.md, the manager and PM role docs where they describe planned/held/awaiting-landing.
Files likely: crates/bridle-daemon (tasks.rs, store schema + migration, server.rs), crates/bridle-api/src/types.rs (wire change, additive: update daemon, CLI, TUI and gateway together), crates/bridle/src (task commands, queue/status output), crates/bridle-tui, crates/bridle-gateway, docs above.
Acceptance: just check passes; tests for hold/unhold (not claimable while held, event and thread recorded), the built state (reached by the existing done/park flow, left by landing), queue output, and the migration (planned with a branch becomes built; planned without stays; rerun is a no-op).
Size: if this grows past one branch's worth, STOP after items 1, 3 and the migration of branch-holding tasks, and say on the task what remains (the UI polish can be a follow-up).
Model: Sonnet. Out of scope: the review-slice hold decisions themselves, kzw2 (structured task fields in general).

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
