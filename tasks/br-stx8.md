+++
id = "br-stx8"
title = "A task's state says what's really happening: held and built-awaiting-landing are states, not 'planned' with a note"
kind = "feature"
state = "planned"
created_at = "2026-10-06T02:04:20.046Z"
updated_at = "2026-10-06T02:05:10.883690Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: stx8
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
