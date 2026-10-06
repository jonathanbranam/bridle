+++
id = "br-vk3y"
title = "Tickets and tasks link each other in metadata, every time: task new takes the ticket, a rule for every creator, the PM traces unlinked tasks"
kind = "feature"
state = "planned"
created_at = "2026-10-06T00:10:01.851Z"
updated_at = "2026-10-06T00:21:44.929839Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: vk3y
Ticket (the ask with the human's words; read first, and RE-READ it when you start: the human's message was cut off and the bridle-ui aide may add to it): docs/tickets/open/tickets-and-tasks-link-each-other-in-metadata-every-time-tas-vk3y.md
Goal: tickets and tasks link each other in metadata, both ways, however the task was made.
Build:
1. `bridle task new --ticket <id>`: writes both links the way `bridle ticket task` does (the ticket's `tasks:` front matter and the task body's `original id:` first line; reuse that code, don't duplicate it). Without --ticket, print the warning "Ticket not provided. Please provide the ticket number." (a warning, not an error; exit 0). Update `bridle task new --help`, docs/design/cli.md.
2. Rules in workflow/base/rules (new file, e.g. ticket-links.md, short, with the human's why): every role that creates tasks or tickets (aides, orchestrator, PM, any other) names the ticket when creating a task. Reference it from the role docs that create tasks (workflow/base/roles/*.md for aide, orchestrator, project-manager) only where those docs already list task-creation steps.
3. A PM rule (same file or the project-manager role doc): a task that arrives without a ticket gets traced to where it came from; if a ticket exists, link it (`bridle ticket set <id> tasks ...` and fix the task body), else make one with `bridle ticket new --from-task`; and record each case so we can see how unlinked tasks keep happening: a task comment "unlinked at creation by <creator>; linked to <ticket>" (greppable, no new store).
Out of scope here: the backfill of today's unlinked pairs is br-e7e2 (v3dk slice B); check that task's brief and, if it does not cover the 48 ticket warnings from `bridle ticket check`, say so in your done note rather than building it. Also out of scope: br-bdrc (who filed a ticket).
Migration: the rules reach projects through workflow sync; no project file changes. State that in the done note.
Acceptance: just check passes; tests for `task new --ticket` (both links written, bad ticket id errors, no flag warns and still creates); docs and rules updated.
Model: Sonnet.

## Thread

### note · agent:pm-1 · 2026-10-06T00:10:35.912Z
PM (pm-1): planned as the core of the ask (task new --ticket + warning, creator rule, PM rule with a task-comment record). The human's message was cut off; the aide is adding the rest to the ticket, so the worker must re-read the ticket at start, and I will amend this brief if the additions change it. Backfill is br-e7e2's, not this task's. Queued in tier 1 beside br-dxcw (different files).

### note · external:aide · 2026-10-06T00:21:44.929Z
From the human, via bridle-ui's aide (2026-10-05 ~8:20 PM ET), the rest of vk3y, now in the ticket: "We should have a --ticket option when creating a task that links it back to the ticket it came from ... Maybe make --ticket required as a CLI parameter. If there's really no ticket, the agent has to enter some sentinel value ... maybe no-ticket ... We should make them always specify a ticket, and then say give some special value if there's no ticket." And: "It should always be linked both ways." This changes pm-1's plan: --ticket is required (with a sentinel such as no-ticket), not optional with a warning.
