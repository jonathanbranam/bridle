+++
id = "br-sehn"
title = "User-facing rename slice 2: roles, rules, skills and docs say ticket, not task (q8cs)"
kind = "feature"
state = "integrated"
created_at = "2026-10-11T09:11:58.800Z"
updated_at = "2026-10-11T10:42:59.481931Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
    "agent:wsehn",
]
branch = "bridle/wsehn"
commit = "a416416706ab9b396f85eacfed5ef370c9334394"
summary = """Slice 2 of the user-facing rename. workflow/base roles, rules and skills now say ticket (prose and commands: ticket comment/show/list/plan/done/summary/land/claim/ask/edit/new). Rule file names are unchanged (talk-on-the-task, report-task-failures). Deliberately still `bridle task`: `new --for-human/--from` and `ready`/`ready --all`, because `bridle ticket` has no form of them (slice 1 note). Where a ticket has a file and a row, rules/tickets.md, manager, PM and worker now say "file" and "row". Design docs: agent-host/*, storage.md and a few cli.md lines swap commands; cli.md's command table was left (slice 1 already notes the alias). Two tests asserting role text (orchestrator prime aide, config manager prompt) updated to the new wording. CHANGELOG line added. Migration: role and rule text reaches projects through the normal workflow sync. Remaining `bridle task` in workflow/base are only the unfolded forms above. Not touched: gateway, code names, alias removal."""
ticket = "sehn"
parent = "br-q8cs"
+++

docs/tickets/open/user-facing-rename-slice-2-roles-rules-skills-and-docs-say-t-sehn.md

Ticket: docs/tickets/open/user-facing-rename-bridle-ticket-replaces-bridle-task-hidden-q8cs.md (read The ask and 22ab section 9). Slice 2 of 3, after br-q8cs (CLI slice) is integrated; the bridle ticket forms must exist before the docs name them. Goal: rename the user-facing words once, against the commands as they now are: roles, rules and skills in workflow/base/ (bridle-worker: claim a task becomes claim a ticket; manager, project-manager, orchestrator, advisor, aide roles; rules such as talk-on-the-task keep their file names but say ticket), and docs: docs/README.md, docs/design/cli.md prose, docs/design/agent-host/, storage.md where it describes what users see. Keep code-level names (tasks table, tasks/ folder, types) as they are: those are step 9. Use the bridle ticket commands in every example. Migration: role and rule text reaches projects through the normal workflow sync; state it in the done note. Acceptance: just check passes; grep of workflow/base for bridle task finds only deliberate mentions of the hidden alias. Model: Sonnet. Out of scope: gateway, code rename, alias removal.

## Thread

### note · agent:wsehn · 2026-10-11T10:42:47.665Z
done: just check exit 0, 1512 tests run (1512 passed, 6 skipped), sha e597391e on bridle/wsehn (main already merged). Roles/rules/skills/docs say ticket; remaining bridle task in workflow/base: new --for-human/--from and ready (no ticket form yet). Role/rule text reaches projects via the normal workflow sync.

### note · agent:wsehn · 2026-10-11T10:42:49.096Z
done: roles, rules, skills and docs say ticket; e597391e, check exit 0, 1512 tests

### note · agent:manager-2 · 2026-10-11T10:42:56.140Z
integrated: a416416706ab9b396f85eacfed5ef370c9334394 (branch bridle/wsehn)

### note · agent:manager-2 · 2026-10-11T10:42:59.481Z
cleanup: removed nothing; kept agent wsehn (background job pid 48713)
