+++
id = "br-g8pe"
title = "Link-reminder hook: design pass (designer): how hooks work, options and a recommendation in ticket 9dcz"
kind = "research"
state = "integrated"
created_at = "2026-10-10T17:09:43.632Z"
updated_at = "2026-10-10T18:22:38.140657Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
size = "M"
branch = "bridle/dg8pe"
commit = "8f3dd0df85f7a54499b1081f90384158dcf0fc94"
summary = "Wrote '## Design' into ticket 9dcz (docs only). Explains hooks plainly; recommends a Stop-hook command 'bridle link-check' in the orchestrator/advisor/aide sessions (once via stop_hook_active, fail open, local ticket-file lookup now, 22ab later); render-time UI links as a separate option; finding: a rule with no roles: line already applies to every role, and the advisor was already listed, so the cause is context drift. 5 questions for the human. Ticket check: the only 9dcz complaint is a pre-existing 'tasks names br-g8pe' linkage message."
+++

Role: designer (workflow/base/roles/designer.md). Ticket: docs/tickets/open/agents-are-reminded-once-to-link-the-ticket-and-task-ids-the-9dcz.md. Read all of it, the human's words especially, and jmng (resolved) for the earlier attempt.

Write '## Design' into the ticket covering the six points under '## The design ask': how Claude Code hooks work, explained plainly for the human (cite the Claude Code hooks docs; look at how bridle already installs `bridle kill-guard` and `bridle focus gate`); where the human reads IDs and which a hook can cover; the check (task IDs always, a bare 4-char token only if that ticket exists; already-linked; speed; a batch bridle command now, the 22ab table later); one reminder, never a loop or a block (stop_hook_active, fail open on error or timeout); the link rule for every role without listing roles; options, a recommendation and questions for the human.

The human wants to read and understand it, so write for a reader who doesn't know hooks. Build nothing. Report on the task; the PdM sends the design to the human. Not urgent: don't preempt other work, but the human would like it today.

## Thread

### note · external:advisor/product-manager · 2026-10-10T17:09:53.880Z
advisor/product-manager (PdM): the human, 2026-10-10 ~1:15 PM ET: "It shouldn't preempt other work, but it is something that's a continual problem, so it'd be nice to get it in today." Please tier it after the current Tier 1-2 work (ygkc, bcw6) but so a designer slot can take it today; it is docs-only (the ticket). The build task br-9dcz stays pending until the human reviews the design.

### note · agent:pm-1 · 2026-10-10T17:09:57.694Z
pm-1: model Sonnet. Docs-only: edit the ticket only, no code. Migration: none (design).

### note · agent:dg8pe · 2026-10-10T18:22:24.016Z
done: '## Design' written into ticket 9dcz (option B Stop hook recommended; 5 questions for the human); a8a0cd6c

### note · agent:manager-2 · 2026-10-10T18:22:32.737Z
integrated: 8f3dd0df85f7a54499b1081f90384158dcf0fc94 (branch bridle/dg8pe)

### note · agent:manager-2 · 2026-10-10T18:22:38.140Z
cleanup: removed agent dg8pe, branch bridle/dg8pe
