+++
id = "br-avu7"
title = "vk3y slice 2: 'bridle task new' requires --ticket (no-ticket sentinel) and a rule for every task creator"
kind = "feature"
state = "pending"
created_at = "2026-10-06T00:37:58.516Z"
updated_at = "2026-10-06T00:44:56.546687Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

original id: vk3y
Ticket: docs/tickets/open/tickets-and-tasks-link-each-other-in-metadata-every-time-tas-vk3y.md (read all of it, and re-read when you start). Depends on br-vk3y (slice 1: the real `ticket` task field and its migration); start only after it merges.
Build:
1. `bridle task new` takes a REQUIRED `--ticket <id>`. With a real ticket id it sets the task's `ticket` field and adds the task to the ticket's `tasks:` front matter, the same way `bridle ticket task` does (reuse that code). With no real ticket the agent passes the sentinel `--ticket no-ticket` (plain, shell-safe, no quoting), stored in the `ticket` field. Omitting --ticket is an ERROR whose message names the sentinel. An unknown ticket id is an error. Update `bridle task new --help` and docs/design/cli.md.
   Breaking change, so find every caller: grep workflow/, docs/, role prompts, scripts, justfile and tests for `task new` and update them to pass --ticket. The CLI change reaches agents on the next daemon upgrade; updated prompts reach projects through workflow sync; no project files change. State this in the done note.
2. Rule in workflow/base/rules (new file, e.g. ticket-links.md, short, with the human's why): every role that creates tasks or tickets (aides, orchestrator, PM, any other) makes a task from its ticket (`bridle ticket task <id>` preferred, `bridle task new --ticket <id>` otherwise) so the link goes both ways; `no-ticket` only when there really is none, loud on purpose. Reference it from the role docs that already list task-creation steps (workflow/base/roles/*.md: aide, orchestrator, project-manager).
Dropped by the human (comment c1 on the ticket, "defined this error away"): NO PM rule for tracing tasks without a ticket. Do not add one.
Out of scope: the backfill of old unlinked pairs (br-e7e2); br-bdrc.
Acceptance: just check passes; tests: both links written, sentinel stored, missing flag errors naming no-ticket, bad id errors; docs, prompts and rules updated; no remaining `task new` example without --ticket.
Model: Sonnet.
