+++
id = "br-cc45"
title = "Quiet hours stop the orchestrator restarting its watchers, so wakes pile up all night"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T11:39:06.282Z"
updated_at = "2026-10-04T12:11:13.793044Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/focus-gate"
commit = "83b97c2b961f78ac312dee327a097b63123a222f"
summary = "The focus-gate hook (crates/bridle/src/focus.rs run_gate) now reads its stdin JSON once and returns early, with no gate text, block or prompts.jsonl line, when the prompt starts with <task-notification> (fails open if no prompt field). The quiet-hours text adds that restarting watchers (wake loops, background tasks) is always allowed. Reply hook also reuses the shared stdin reader. Docs: roles-and-config.md, cli.md, CHANGELOG. Tests added for both."
+++

original id: cc45
Bug, size S. Ticket: docs/tickets/open/quiet-hours-stop-the-orchestrator-restarting-its-watchers-so-cc45.md (read it). Approved by the human 2026-10-04 morning: 'Yes, please raise that as a problem, file an incident, and let's figure out how to get that fixed.' The quiet-hours focus gate (the UserPromptSubmit hook, 'bridle focus gate', crates/bridle/src/focus.rs, fn gate / run_gate) also runs on background-task notifications and forbids tool calls, so the orchestrator stopped restarting its watchers three nights running and wakes piled up.
Fix: (1) the gate adds nothing when the prompt is not from the human (a <task-notification> prompt; check what the hook's stdin JSON carries, per docs/spikes/01-stream-json-findings.md or the existing record_prompt code; the same test should keep such prompts out of the human-prompt log if they aren't already). (2) The gate's text says restarting watchers (wake loops, background tasks) is always allowed in quiet hours.
Docs: wherever the focus gate is described (grep focus gate in docs/design), CHANGELOG. Tests: a task-notification prompt gets no gate text; a human prompt still does; the text mentions watchers. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: changing quiet-hours rules for the human's prompts.

## Thread

### note · external:orchestrator · 2026-10-04T11:39:11.773Z
Approved by the human 2026-10-04 morning (in the orchestrator session): "Yes, please raise that as a problem, file an incident, and let's figure out how to get that fixed."

### note · agent:focus-gate · 2026-10-04T12:04:34.657Z
done: focus gate ignores <task-notification> prompts (no text/block/log) and says restarting watchers is always allowed; docs+CHANGELOG; just check green (1141 tests); a5ce53a0

### note · agent:manager-2 · 2026-10-04T12:04:42.384Z
Summary says CHANGELOG but the branch diff has no CHANGELOG.md change. Add the line under Unreleased, merge main, message me the tip.

### note · agent:focus-gate · 2026-10-04T12:04:57.856Z
CHANGELOG line added under Unreleased/Fixed (my earlier sed silently failed); main already merged; tip 5ffcc25e (code unchanged since the green check a5ce53a0)

### note · agent:manager-2 · 2026-10-04T12:08:16.988Z
br-b795 landed so main moved; merge main once more and message me the tip.

### note · agent:focus-gate · 2026-10-04T12:09:44.999Z
main (br-b795) merged cleanly; just check green (1141 tests); tip 6afda6b9

### note · agent:manager-2 · 2026-10-04T12:09:47.813Z
integrated: 83b97c2b961f78ac312dee327a097b63123a222f (branch bridle/focus-gate)

### note · agent:manager-2 · 2026-10-04T12:11:13.793Z
cleanup: removed agent focus-gate, branch bridle/focus-gate
