+++
id = "br-vk3y"
title = "Tasks get a real 'ticket' field, replacing the 'original id:' first body line, with an automatic migration (vk3y slice 1)"
kind = "feature"
state = "planned"
created_at = "2026-10-06T00:10:01.851Z"
updated_at = "2026-10-07T05:11:02.834679Z"
created_by = "external:aide"
watchers = ["external:aide"]
summary = 'Tasks now carry `ticket: Option<String>` (front-matter `ticket = "<id>"`; no SQLite column), added to the Task type, NewTaskRequest, EditTaskRequest (API, additive), the gateway TaskDetail and its regenerated TS binding, and `task show`. `ticket task`/`ticket new --from-task` set it and write no body line; `task edit` never touches it (keep_origin_line removed); `ticket check` reads it (falling back to the old line from an unmigrated daemon). TaskManager::open migrates each task whose first body line is `original id: <x>` (field set, line removed, written through the normal state-branch enqueue); idempotent. An old client sending the line on create is converted the same way. Docs: storage.md, cli.md, README.md, CHANGELOG. Not done: UI linking (the gateway only exposes the id; bridle-ui lives elsewhere).'
+++

original id: vk3y
Ticket (the ask with the human's words; read all of it, including "The human on `original id:`"; RE-READ it when you start). Also read ticket kzw2, section "Decided: the `ticket` field" (docs/tickets/open/task-data-the-system-acts-on-is-structured-fields-not-text-t-kzw2.md): it overrides any older mention of a column. Ticket vk3y: docs/tickets/open/tickets-and-tasks-link-each-other-in-metadata-every-time-tas-vk3y.md
This is slice 1 of 2. Slice 2 (br-avu7, depends on this one) makes `bridle task new --ticket` required with the no-ticket sentinel, and adds the creator rule. Do not build slice 2 here.
Goal: the task -> ticket link is a real task field named exactly `ticket`, not free text. Today it is the first line of the task body (`original id: <ticket>`; see docs/design/storage.md ~71, docs/design/cli.md ~251 and ~442, crates/bridle-daemon/src/tasks.rs ~494 and ~1582).
DECIDED by the human (2026-10-05, via advisor fields): the field is a FRONT-MATTER key `ticket = "<ticket id>"` in the task file, like the other task front-matter fields. NO SQLite column and NO schema migration: the daemon holds every task in memory (TaskManager::open hydrates the cache from the state branch; tasks.rs ~101, ~160), so "tasks for ticket X" is a scan of that cache. Do not add an index or a column.
Build:
- Add `ticket: Option<String>` to the task front matter and in-memory task (tasks.rs), exposed as "ticket" in `bridle task show`, in the API (crates/bridle-api/src/types.rs: additive optional field; update daemon, CLI, TUI and gateway together) and in the gateway/UI bindings (crates/bridle-gateway/bindings/*.ts; show it where a task is displayed, linked to the ticket if the UI already links tickets).
- `bridle ticket task`, `bridle ticket new` (when it files a task) and any other code that writes the `original id:` line today set the field instead and stop writing the body line. `task edit` must keep the field (today it preserves the body line).
- `bridle ticket check` and every other reader of the `original id:` line read the field instead (during migration, also the old line).
- MIGRATION (required, automatic, idempotent): at daemon start, for each existing task whose body's first line is `original id: <x>`, set front-matter `ticket = "<x>"` and remove that line from the body; write it back through the normal task write path so it reaches the bridle/state branch (crates/bridle-daemon/src/state_branch.rs). Tasks without the line are left alone. A second run changes nothing. No per-project file changes. Document in docs/design/storage.md (task front matter and migration) and cli.md.
Out of scope: required --ticket and the sentinel (slice 2); the backfill of unlinked old pairs (br-e7e2; say in your done note whether the 48 `ticket check` warnings are covered there); br-bdrc.
Acceptance: just check passes; tests: field round-trips through the task file, store/cache and API, `ticket task` sets it and writes no body line, `task edit` keeps it, the migration converts a body with the line (and is a no-op on rerun), ticket check reads the field; docs updated.
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

### note · external:aide · 2026-10-06T00:46:01.849Z
From the human, via aide (2026-10-05 ~9:05 PM ET): "Pause br-vk3y while we work on the swish". ("the swish" is likely dictation for "this": the human and I are discussing new ticket jmng, every task and ticket ID the human sees made a clickable link automatically, which overlaps vk3y. Not yet asked to schedule jmng.) Please hold br-vk3y; don't start a worker on it until the human says.

### question · external:orchestrator · 2026-10-06T00:46:23.955Z
ON HOLD by the human (2026-10-05 ~9:05 PM ET, via aide): 'Pause br-vk3y while we work on this.' They are working out ticket jmng (every task/ticket ID shown becomes a link), which overlaps. Don't start a worker. The orchestrator answers this when the human says go.

### note · external:aide · 2026-10-06T00:57:30.602Z
From the human, via aide (2026-10-05 ~9:45 PM ET): "Before br-vk3y before that ticket is worked, I want to resolve this question about original ID. I think it should be converted to front matter, and the name "original ID" is not good. It should be "ticket ID," but I want to see that clearly written up." Held until the human reviews ticket kzw2 (task fields, structured not text), where the full quote is.

### note · external:advisor/fields · 2026-10-06T02:42:19.019Z
From the human, via advisor fields (2026-10-05): the ticket link decision is made, recorded in ticket kzw2 ("Decided: the `ticket` field"). The human: "Sure, I'm fine with just a ticket." The brief needs changing before it is built: the link is a front-matter field `ticket = "<id>"` in the task file, with NO SQLite column (the daemon's in-memory task cache already makes lookup by ticket fast). The rest stays: shown in task show, the API and the UI; ticket new/ticket task set it and stop writing the `original id:` line; ticket check reads the field; a migration moves each `original id:` first line into the field and removes the line. The hold is still yours to release when the human says go.

### note · agent:pm-1 · 2026-10-06T02:42:41.680Z
pm-1: brief amended per advisor fields' comment: the ticket link is a front-matter field, no SQLite column, no schema migration; the migration moves 'original id:' lines into the field at daemon start. Still HELD until the human says go.

### note · external:aide · 2026-10-07T00:59:10.478Z
From the human, via aide (2026-10-06 ~9:00 PM ET): "yes for br-vk3y - but only for the ticket field, not other fields". Hold released for the ticket field ONLY: build the `ticket` front-matter field (kzw2), no other new fields.

### answer · external:orchestrator · 2026-10-07T00:59:32.602Z
Hold released by the human, via aide (2026-10-06 ~9:00 PM ET): 'yes for br-vk3y - but only for the ticket field, not other fields'. Build the brief as written (the ticket front-matter field, its display, ticket task/new writing it, ticket check reading it, the migration). No other new fields; kzw2's other fields stay out.
