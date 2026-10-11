+++
id = "br-sehn"
title = "User-facing rename slice 2: roles, rules, skills and docs say ticket, not task (q8cs)"
kind = "feature"
state = "planned"
created_at = "2026-10-11T09:11:58.800Z"
updated_at = "2026-10-11T09:12:55.027064Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "external:advisor/product-manager",
]
ticket = "sehn"
parent = "br-q8cs"
+++

docs/tickets/open/user-facing-rename-slice-2-roles-rules-skills-and-docs-say-t-sehn.md

Ticket: docs/tickets/open/user-facing-rename-bridle-ticket-replaces-bridle-task-hidden-q8cs.md (read The ask and 22ab section 9). Slice 2 of 3, after br-q8cs (CLI slice) is integrated; the bridle ticket forms must exist before the docs name them. Goal: rename the user-facing words once, against the commands as they now are: roles, rules and skills in workflow/base/ (bridle-worker: claim a task becomes claim a ticket; manager, project-manager, orchestrator, advisor, aide roles; rules such as talk-on-the-task keep their file names but say ticket), and docs: docs/README.md, docs/design/cli.md prose, docs/design/agent-host/, storage.md where it describes what users see. Keep code-level names (tasks table, tasks/ folder, types) as they are: those are step 9. Use the bridle ticket commands in every example. Migration: role and rule text reaches projects through the normal workflow sync; state it in the done note. Acceptance: just check passes; grep of workflow/base for bridle task finds only deliberate mentions of the hidden alias. Model: Sonnet. Out of scope: gateway, code rename, alias removal.
