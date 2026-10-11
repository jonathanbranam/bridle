+++
id = "br-ft3b"
title = "Per-role handover instructions in the workflow, with project overrides"
kind = "feature"
state = "planned"
created_at = "2026-10-05T10:25:03.907Z"
updated_at = "2026-10-11T14:23:04.018393Z"
created_by = "external:orchestrator@nuc"
watchers = [
    "external:orchestrator@nuc",
    "external:advisor/product-manager",
]
summary = """Handover request text is now workflow content. New `crates/bridle-daemon/src/handover.rs`: `workflow/base/handover/standard.md` + appended `<role>.md` (advisor, aide, orchestrator, manager, worker, project-manager); a project's `.bridle/handover/<role>.md` replaces both; no workflow root falls back to the built-in standard text (include_str). Used by `bridle session restart --handover` (`handover_request` in crates/bridle/src/session.rs; hard limit and nightly [[restart]] go through it, reason sentence kept in front) and the agent `Context handoff:` note (supervisor.rs). Standard text calls out what a note cannot summarise; advisor/orchestrator parts ask for next wake-up time and the day's plan state. Role = identity before "/". Project files are read from the CLI's cwd (caveat: a `--project` run from outside the repo gets the built-in text). Override uses a new `.bridle/handover/` dir rather than `.bridle/roles/<role>.md` because that one appends to the system prompt and cannot replace. Migration: base text arrives via the workflow checkout/sync; projects with no override just get the improved standard text. Tests: 4 in handover.rs (standard, extends, replaces, built-in) + 1 in session.rs (reason kept, same resolver). Docs: orchestrator-supervision.md section 7, CHANGELOG."""
ticket = "ft3b"
+++

Ticket: docs/tickets/open/per-role-handover-instructions-in-the-workflow-with-project-ft3b.md (read the ask; the thread has the human's two requests). Human-approved 2026-10-11: build it AFTER the everything-is-a-ticket epic; do not start until br-res3 is integrated (blocked_by in the ticket).

Goal: the text of a handover request is part of the workflow system, per role, with project overrides. Today the request text is hard-coded in the daemon (find it: grep handover in crates/bridle-daemon/src/sessions.rs, orchestrator.rs, and crates/bridle/src/session.rs; br-cbbn added a reason parameter in crates/bridle-daemon/src/restart_at.rs). Build: (1) a standard handover text in workflow/base (how to hand over and the prompt the daemon sends), (2) a per-role part (orchestrator, advisor, aide, manager, worker, project-manager) that replaces or extends it, (3) project layering the same way role docs layer today (.bridle/roles/<role>.md or a handover section/file: look at how role docs and overrides are resolved and use that mechanism, do not invent a second one), (4) every handover path uses it: context-driven handover (gq9r), the nightly restart (cbbn, with its reason sentence kept), and session restart --handover. Example need: the notes advisor handover must state explicitly the wake-up time tomorrow and the day's plan state; and the human wants things that cannot be summarised in a note (what is in context) called out in the instructions. Keep it simple: text files, no new config.
Files: workflow/base/ (standard handover text, role parts; add the base version to whatever list sync uses), the role resolver in crates/bridle (workflow code), the daemon handover paths above, docs/design/agent-host/ (handover and workflow docs), CHANGELOG.
Migration: base text reaches projects through workflow sync; project overrides are optional new files; a project with none behaves as today apart from the improved standard text. State it in the done note.
Acceptance: just check passes; tests: a role with no override gets the standard text; a role part extends it; a project override replaces it; the nightly restart request still contains its reason and time; session restart --handover uses the same resolver.
Model: Sonnet. Out of scope: other prompts, a UI for editing, per-session overrides.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T10:25:03.907Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T10:25:23.857Z
Triage (pm-1): accept (the human's own ask). Ticket minted (uncommitted; commit on main). Stays pending until approved with `bridle task ready br-ft3b`; then I plan it (Sonnet). Suggested order: after 4s3z and gq9r (it feeds gq9r's handover text, and cbbn).

### note · external:advisor/product-manager · 2026-10-09T11:04:38.766Z
watching the task

### note · external:orchestrator · 2026-10-11T12:55:51.535Z
human via orchestrator@nuc (from the notes advisor, m-0151), the human, 2026-10-10: "I think there's a lot of stuff in your context that can't be summarized effectively in a handoff note, and I want that part of your handoff instructions in the future. I'm not sure if we have that capability, like custom handoff instructions, but we need to have that". Second time the human has asked for this (first 2026-10-05, in the ask).

### note · external:advisor/product-manager · 2026-10-11T13:15:23.680Z
human via advisor/product-manager, 2026-10-11 ~9:00 AM ET: "we need ft3b, yes, as long as it doesn't interrupt / and or comes AFTER everything is a ticket, that is good." Readied; blocked_by br-res3 (the last 22ab code-rename slice). pm-1: plan it now; no worker before br-res3 lands.

### note · agent:wft3b · 2026-10-11T14:22:57.287Z
done: per-role handover instructions in the workflow with project overrides; just check exit 0, 1523 tests, 17b91cb8

### note · agent:wft3b · 2026-10-11T14:23:00.842Z
done: handover instructions as workflow files; just check exit 0, 1523 tests passed, sha 17b91cb8. See the summary.

### note · agent:manager-2 · 2026-10-11T14:23:04.018Z
main moved. Merge main, re-run just check, message me the sha right away; I will land immediately.
