+++
id = "br-vk3y"
title = "Tasks get a real 'ticket' field, replacing the 'original id:' first body line, with an automatic migration (vk3y slice 1)"
kind = "feature"
state = "planned"
created_at = "2026-10-06T00:10:01.851Z"
updated_at = "2026-10-06T00:38:01.886388Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: vk3y
Ticket (the ask with the human's words; read all of it, including "The human on `original id:`"; RE-READ it when you start in case more is added): docs/tickets/open/tickets-and-tasks-link-each-other-in-metadata-every-time-tas-vk3y.md
This is slice 1 of 2. Slice 2 (new task, depends on this one) makes `bridle task new --ticket` required with the no-ticket sentinel, and adds the creator rule. Do not build slice 2 here.
Goal: the task -> ticket link is a real task field named `ticket`, not free text. Today it is the first line of the task body (`original id: <ticket>`; see docs/design/storage.md ~71, docs/design/cli.md ~251 and ~442, crates/bridle-daemon/src/tasks.rs ~494 and ~1582; "original" because a ticket's task takes its id from the ticket, vk3y -> br-vk3y).
Build:
- A `ticket` column on tasks (nullable; the value is a ticket id, or the sentinel `no-ticket` which slice 2 will write), exposed as "ticket" in `bridle task show`, the API (crates/bridle-api/src/types.rs: a wire change, update the daemon, CLI, TUI and gateway together; additive, optional field) and the gateway/UI bindings (crates/bridle-gateway/bindings/*.ts, show it where a task is displayed; link it to the ticket if the UI already links tickets).
- Set by `bridle ticket task`, `bridle ticket new` (when it files a task) and any other code that writes the `original id:` line today; those stop writing the body line.
- `bridle ticket check` and any other reader of the `original id:` line read the field instead (and, during migration, the old line).
- MIGRATION (required, automatic): a normal DB schema migration at daemon start that parses an `original id: <x>` first line from each existing task body into the field and strips that line from the body. Idempotent; tasks with no such line get NULL. Covers every project's daemon when it upgrades; no per-project file changes. If tasks are also mirrored to the bridle/state branch (crates/bridle-daemon/src/state_branch.rs), check that the export carries the field and re-imports cleanly. Document the migration in docs/design/storage.md.
Out of scope: required --ticket and the sentinel behaviour, the creator rule (slice 2); the backfill of unlinked old pairs (br-e7e2; say in your done note whether the 48 `ticket check` warnings are covered there); br-bdrc.
Acceptance: just check passes; tests: field round-trips through the store and API, `ticket task` sets it and writes no body line, the migration converts a body with the line (and is a no-op on rerun), ticket check reads the field; docs updated (storage.md, cli.md).
Model: Sonnet.

## Thread

### note · agent:pm-1 · 2026-10-06T00:10:35.912Z
PM (pm-1): planned as the core of the ask (task new --ticket + warning, creator rule, PM rule with a task-comment record). The human's message was cut off; the aide is adding the rest to the ticket, so the worker must re-read the ticket at start, and I will amend this brief if the additions change it. Backfill is br-e7e2's, not this task's. Queued in tier 1 beside br-dxcw (different files).

### note · external:aide · 2026-10-06T00:21:44.929Z
From the human, via bridle-ui's aide (2026-10-05 ~8:20 PM ET), the rest of vk3y, now in the ticket: "We should have a --ticket option when creating a task that links it back to the ticket it came from ... Maybe make --ticket required as a CLI parameter. If there's really no ticket, the agent has to enter some sentinel value ... maybe no-ticket ... We should make them always specify a ticket, and then say give some special value if there's no ticket." And: "It should always be linked both ways." This changes pm-1's plan: --ticket is required (with a sentinel such as no-ticket), not optional with a warning.

### note · agent:pm-1 · 2026-10-06T00:22:09.612Z
PM (pm-1): brief amended for the human's follow-up (now in the ticket): --ticket is required with the sentinel no-ticket; omission is an error naming it; links both ways; callers across workflow/docs/prompts must be updated since this breaks plain `task new`. Note: I (the PM) and the orchestrator will need to pass --ticket on every `task new` once this lands.

### note · external:aide · 2026-10-06T00:25:53.609Z
From the human, via bridle-ui's aide (2026-10-05 ~8:30 PM ET), comment c1 on ticket vk3y, on the PM tracing rule: "We can remove this since we have 'defined this error away'". The ticket is edited (584f6668): no PM tracing rule; --ticket is required, with no-ticket as the explicit sentinel. Please drop the PM rule from br-vk3y's brief before it's built.

### note · agent:pm-1 · 2026-10-06T00:26:15.325Z
PM (pm-1): brief amended: the PM tracing rule is removed, per the human's comment c1 on the ticket. What remains: required --ticket with the no-ticket sentinel, and the creator rule.

### note · external:aide · 2026-10-06T00:37:24.321Z
From the human, via bridle-ui's aide (2026-10-05 ~8:45 PM ET), added to ticket vk3y: "What is original ID in the task? Is that a field? ... If that's the ticket ID, then just say ticket ID, not original ID." Today it's the first body line (text, not metadata). vk3y now asks for a real task field 'ticket' (task show, API, UI), with existing 'original id:' lines migrated. Please add this to br-vk3y's brief before it's built.

### note · agent:pm-1 · 2026-10-06T00:38:01.886Z
PM (pm-1): brief rewritten for the human's addition (a real `ticket` field plus an automatic migration of `original id:` lines). It is now slice 1 of 2: the field, API, UI display and migration. The required --ticket flag and the creator rule moved to a new task, br-avu7 (pending; needs `bridle task ready br-avu7`, then I plan it with an edge on this one). PM tracing rule stays dropped.
