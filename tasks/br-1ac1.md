+++
id = "br-1ac1"
title = "Tasks record their landing: branch, merge commit and implementation summary (tr7k)"
kind = "feature"
state = "planned"
created_at = "2026-09-29T01:38:35.918Z"
updated_at = "2026-09-29T01:38:37.577095Z"
+++

Goal: bridle task show prints, for a finished task, the branch that did it, the commit that landed it and a short summary of how it was implemented. Ticket: docs/questions/open/task-landing-record-tr7k.md (human's words in squash-merge-each-task-sq4m.md). Today bridle task done <id> --commit <sha> (br-789a) records only the commit in the thread. Do: (1) add --branch <name> to task done, recorded with the commit; (2) let a worker record the summary on the task at handoff, smallest design that works (recommended: a bridle task summary <id> with --text or --file path-or-dash, stored on the task record and state branch like size was in br-0685; replacing an earlier one); (3) task show prints branch, commit and summary together; (4) task done warns but does not fail when no summary exists. Read docs/design/storage.md and the tasks docs; update them and docs/design/cli.md; check the state-branch round trip and rebuild if present. Also update the worker role and skill (write the summary before reporting done) and manager role and skill (record --branch and --commit; do not mark done without a summary) under workflow/base/. Acceptance: just check passes; tests for the new flag, summary set and replace, and show output. Model: Sonnet. Out of scope: the squash merge itself (next task, sq4m), meta-notes' project-layer prompt (the orchestrator applies that), bridle rm. Must merge before the sq4m task; both edit workflow/base/roles/manager.md.
