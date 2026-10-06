+++
id = "br-vk3y"
title = "Tickets and tasks link each other in metadata, every time: task new takes the ticket, a rule for every creator, the PM traces unlinked tasks"
kind = "feature"
state = "planned"
created_at = "2026-10-06T00:10:01.851Z"
updated_at = "2026-10-06T00:22:09.612493Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: vk3y
Ticket (the ask with the human's words; read first, including "The human's follow-up" section; RE-READ it when you start in case more is added): docs/tickets/open/tickets-and-tasks-link-each-other-in-metadata-every-time-tas-vk3y.md
Goal: tickets and tasks link each other in metadata, both ways, however the task was made.
Build:
1. `bridle task new` takes a REQUIRED `--ticket <id>`. With a real ticket id it writes both links the way `bridle ticket task` does (the ticket's `tasks:` front matter and the task body's `original id:` first line; reuse that code, don't duplicate it). With no real ticket the agent passes the sentinel `--ticket no-ticket` (plain, shell-safe, no quoting), which is recorded on the task (e.g. an `original id: no-ticket` first line, so the PM can find and trace them: make it greppable and listable, e.g. `bridle task list` can filter or the body line is enough; pick the smallest). Omitting --ticket is an ERROR whose message names the sentinel. An unknown ticket id is an error. Update `bridle task new --help` and docs/design/cli.md.
   Breaking change, so find every caller: grep workflow/, docs/, role prompts, scripts, justfile and tests for `task new` and update them to pass --ticket. The CLI change reaches agents on the next daemon upgrade; the updated prompts reach projects through workflow sync; no project files change. State this in the done note.
2. Rule in workflow/base/rules (new file, e.g. ticket-links.md, short, with the human's why): every role that creates tasks or tickets (aides, orchestrator, PM, any other) makes a task from its ticket (`bridle ticket task <id>` is preferred, `bridle task new --ticket <id>` otherwise) so the link goes both ways; `no-ticket` only when there really is none, and it is loud on purpose. Reference it from the role docs that already list task-creation steps (workflow/base/roles/*.md: aide, orchestrator, project-manager).
3. A PM rule (same file or the project-manager role doc): a task carrying `no-ticket` (or otherwise unlinked) gets traced to where it came from; if a ticket exists, link it (`bridle ticket set <id> tasks ...` and fix the task body), else make one with `bridle ticket new --from-task`; record each case so we can see how unlinked tasks keep happening: a task comment "unlinked at creation by <creator>; linked to <ticket>" (greppable, no new store).
Out of scope here: the backfill of today's unlinked pairs is br-e7e2 (v3dk slice B); check that task's brief and, if it does not cover the 48 ticket warnings from `bridle ticket check`, say so in your done note rather than building it. Also out of scope: br-bdrc (who filed a ticket).
Acceptance: just check passes; tests for `task new --ticket` (both links written; sentinel recorded; missing flag errors naming no-ticket; bad ticket id errors); docs, prompts and rules updated; no remaining `task new` example without --ticket.
Model: Sonnet.

## Thread

### note · agent:pm-1 · 2026-10-06T00:10:35.912Z
PM (pm-1): planned as the core of the ask (task new --ticket + warning, creator rule, PM rule with a task-comment record). The human's message was cut off; the aide is adding the rest to the ticket, so the worker must re-read the ticket at start, and I will amend this brief if the additions change it. Backfill is br-e7e2's, not this task's. Queued in tier 1 beside br-dxcw (different files).

### note · external:aide · 2026-10-06T00:21:44.929Z
From the human, via bridle-ui's aide (2026-10-05 ~8:20 PM ET), the rest of vk3y, now in the ticket: "We should have a --ticket option when creating a task that links it back to the ticket it came from ... Maybe make --ticket required as a CLI parameter. If there's really no ticket, the agent has to enter some sentinel value ... maybe no-ticket ... We should make them always specify a ticket, and then say give some special value if there's no ticket." And: "It should always be linked both ways." This changes pm-1's plan: --ticket is required (with a sentinel such as no-ticket), not optional with a warning.

### note · agent:pm-1 · 2026-10-06T00:22:09.612Z
PM (pm-1): brief amended for the human's follow-up (now in the ticket): --ticket is required with the sentinel no-ticket; omission is an error naming it; links both ways; callers across workflow/docs/prompts must be updated since this breaks plain `task new`. Note: I (the PM) and the orchestrator will need to pass --ticket on every `task new` once this lands.
