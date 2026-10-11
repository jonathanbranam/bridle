+++
id = "br-bpku"
title = "Every ticket gets a row; types fix and epic; readiness moves to the ticket (ticket ready)"
kind = "feature"
state = "integrated"
created_at = "2026-10-09T19:47:21.257Z"
updated_at = "2026-10-11T06:50:34.516588Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
branch = "bridle/wbpku"
commit = "be2acb4863635bd12055a857387afa1c496567ef"
summary = "Every ticket now has a row and readiness moved to the ticket. New kinds fix, epic, docs (TaskKind::is_change marks feature/fix/docs/chore/arch-revision; no idea kind). `ticket new` files the row (same id, pending, tasks: set; needs the daemon; --from-task unchanged); `ticket task` and --no-task are gone. `ticket ready <id>` replaces `task ready <id>` for ticketed rows: refuses an empty ask or a ticket not in local main, then readies the row; `task ready <id>` still works only for ticketless rows (ticket submit) and refuses a ticketed one. Migration 0003-bug-tickets-that-are-fixes (opt-in, manual_only, protected-branch guard): kind bug -> fix for tickets that list a task; applied to this repo only (67 files, separate commit). Other projects: owner runs `bridle migrate --only 0003-bug-tickets-that-are-fixes` on a trial branch and reviews. Rows are not migrated (file migration only): this repo's open bug rows and integrated ones keep kind bug; the running daemon is the old binary and would not parse fix, so change open ones with `task kind` after the upgrade. Not done (out of scope per the brief or deferred): per-type workflows/states (stx8), parts and gates (95mu), links (3v75), thread (72t9). Docs: cli.md, storage.md, migrations.md, tasks-and-tickets.md, README, rule tickets, roles manager/orchestrator/project-manager, roles-and-lifecycle, CHANGELOG."
ticket = "bpku"
+++

Ticket: docs/tickets/open/every-ticket-gets-a-row-types-fix-and-epic-readiness-moves-t-bpku.md (read "The ask"; it is the scope). Also read ticket 22ab sections 2 and 3 (the parent plan, human approved).

Order: 22ab steps 1-4 go one at a time (all change the ticket tooling). Starts after br-wrmu (syqn part 3) is integrated; br-3v75 and br-72t9 follow this one. The human wants the epic landed or close by morning: do not wait on anything else.

Size: this is big. If the context nears 150K tokens, land a coherent first slice (e.g. rows for every ticket and the new types) and say on the thread what remains, so the manager can file the rest as a follow-up task; do not leave main red.
Migration: the ticket names it (bug tasks that are fixes become fix); implement it as an xebc migration, trial branches only for other projects (rule existing-projects). State how existing projects are updated in the done note.
Acceptance: just check passes, plus the ticket's items. Model: Sonnet.

## Thread

### note · external:advisor/product-manager · 2026-10-11T04:28:10.462Z
Readied by advisor (product-manager), under the human-approved 22ab plan (2026-10-09: step 2, no human gate, readied when buildable). Step 1's br-syqn is integrated; its parts br-epvy (done, landing) and br-wrmu (the migration) are still in flight. Plan it now so it starts right after br-wrmu lands: 22ab steps 1-4 go one at a time (they all change the ticket tooling). The human wants the epic to land or get close by morning.

### note · agent:wbpku · 2026-10-11T06:35:21.753Z
done: every ticket gets a row, fix/epic/docs kinds, ticket ready, migration 0003 (applied to this repo); just check exit 0, 1501 tests; 127d242e

### note · agent:manager-2 · 2026-10-11T06:50:20.816Z
integrated: be2acb4863635bd12055a857387afa1c496567ef (branch bridle/wbpku)

### note · agent:manager-2 · 2026-10-11T06:50:34.516Z
cleanup: removed agent wbpku, branch bridle/wbpku
