+++
id = "br-cc45"
title = "Quiet hours stop the orchestrator restarting its watchers, so wakes pile up all night"
kind = "bug"
state = "planned"
created_at = "2026-10-04T11:39:06.282Z"
updated_at = "2026-10-04T11:39:19.219581Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: cc45
Bug, size S. Ticket: docs/tickets/open/quiet-hours-stop-the-orchestrator-restarting-its-watchers-so-cc45.md (read it). Approved by the human 2026-10-04 morning: 'Yes, please raise that as a problem, file an incident, and let's figure out how to get that fixed.' The quiet-hours focus gate (the UserPromptSubmit hook, 'bridle focus gate', crates/bridle/src/focus.rs, fn gate / run_gate) also runs on background-task notifications and forbids tool calls, so the orchestrator stopped restarting its watchers three nights running and wakes piled up.
Fix: (1) the gate adds nothing when the prompt is not from the human (a <task-notification> prompt; check what the hook's stdin JSON carries, per docs/spikes/01-stream-json-findings.md or the existing record_prompt code; the same test should keep such prompts out of the human-prompt log if they aren't already). (2) The gate's text says restarting watchers (wake loops, background tasks) is always allowed in quiet hours.
Docs: wherever the focus gate is described (grep focus gate in docs/design), CHANGELOG. Tests: a task-notification prompt gets no gate text; a human prompt still does; the text mentions watchers. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: changing quiet-hours rules for the human's prompts.

## Thread

### note · external:orchestrator · 2026-10-04T11:39:11.773Z
Approved by the human 2026-10-04 morning (in the orchestrator session): "Yes, please raise that as a problem, file an incident, and let's figure out how to get that fixed."
