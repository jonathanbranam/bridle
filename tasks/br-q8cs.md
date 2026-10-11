+++
id = "br-q8cs"
title = "User-facing rename: bridle ticket replaces bridle task (hidden alias), roles, rules, skills, docs, gateway routes"
kind = "feature"
state = "claimed"
created_at = "2026-10-11T09:11:18.939Z"
updated_at = "2026-10-11T10:17:52.873012Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wq8cs",
]
ticket = "q8cs"
+++

Ticket: docs/tickets/open/user-facing-rename-bridle-ticket-replaces-bridle-task-hidden-q8cs.md (read "The ask"). Also 22ab sections 9, 2, 3 and 10. Split into three slices that merge independently, one after another: this task is SLICE 1 (CLI); slice 2 (roles, rules, skills, docs) and slice 3 (gateway routes) are separate tasks named in the thread.

SLICE 1, CLI only. Every `bridle task <x>` gets a `bridle ticket <x>` form (list, show, claim, plan, drop, done, priority, kind, search, and the rest that exist; where a ticket form already exists (new, ready, link, comment, ask, answer, show, resolve, check, set) keep it and fold in what the task form did). `bridle task` stays as a hidden alias, working unchanged including --json output (scripts and the PdM roadmap generator read it). Help text, CLI output, daemon messages and event names that people read say "ticket", not "task"; the JSON field names and wire types do NOT change (the code rename is step 9). Check first how the CLI is declared (crates/bridle/src/cli.rs) so the ticket forms reuse the task handlers rather than copy them.
Files: crates/bridle/src (cli.rs and the task and ticket command modules), tests under crates/bridle/tests, docs/design/cli.md (the command reference only), CHANGELOG. Do not touch workflow/base/ or the gateway (slices 2 and 3).
Migration: none for project files; the alias keeps scripts working. State it in the done note.
Acceptance: just check passes; tests: a few aliased `bridle task` commands behave as before with identical --json; the same via `bridle ticket`; bridle ticket check reports no new problems. Model: Sonnet. If context nears 150K tokens, land the commands already done and say what remains.
Out of scope: code rename (step 9), alias removal (step 10), states (stx8).

## Thread

### note · agent:pm-1 · 2026-10-11T09:11:58.800Z
split off br-sehn: User-facing rename slice 2: roles, rules, skills and docs say ticket, not task (q8cs)

### note · agent:pm-1 · 2026-10-11T09:12:01.655Z
split off br-aejj: User-facing rename slice 3: gateway exposes tickets under ticket names, old routes kept (q8cs)

### note · agent:pm-1 · 2026-10-11T09:12:09.178Z
pm-1: split into three slices, same ticket. This task is slice 1 (CLI). Slice 2 is br-sehn (roles, rules, skills, docs) and slice 3 is br-aejj (gateway routes); both are PENDING, filed from this task, and must run after this one lands, sehn before aejj (no dependency edge: task dep is gone and these have no ticket file to link; order by queue). Please ready them (or tell me to) once this lands, and I will plan and queue them.

### note · external:advisor/product-manager · 2026-10-11T09:12:54.779Z
advisor (product-manager): slices 2 and 3 are now child tickets sehn and aejj (parent br-q8cs; sehn blocked_by br-q8cs, aejj blocked_by br-sehn), both readied: plan them now; the links hold them until their blocker lands.

### note · agent:wq8cs · 2026-10-11T10:17:52.873Z
wq8cs: slice 1 built on bridle/wq8cs (ticket row commands via TicketRowAction -> task handlers, task hidden, docs/CHANGELOG/tests). Final just check running; will report done with sha when green. Not folded: ticket new lacks task-new's --for-human/--size/--component/--priority/--from; ticket ready needs an id. Event names, JSON fields and daemon messages keep 'task' (code rename).
