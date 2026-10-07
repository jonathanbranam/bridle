+++
id = "br-xv2n"
title = "One handover command, slice 2: role files and prime text use 'bridle handover write'"
kind = "chore"
state = "planned"
created_at = "2026-10-07T10:16:38.953Z"
updated_at = "2026-10-07T10:54:02.490510Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Ticket: docs/tickets/open/one-handover-command-the-same-for-every-agent-3kdc.md. Slice 2 of 2; blocked on br-3kdc (slice 1) AND the daemon running a build with it (the human: "after it's changed and delivered, the role files get updated for Orchestrator"). Start only when the orchestrator says slice 1 is delivered.
Goal: every role file and the prime text tell the agent to hand over with the one command, `bridle handover write --file -` (it records the note and restarts the writer's own session; `--no-restart` for note-only), not `bridle orchestrator handover write/done`, `bridle orchestrator prime orchestrator` handover steps or `bridle session restart --handover` as a self-handover.
Files: workflow/base/roles/*.md (orchestrator first, then aide, advisor, managers, others that mention handover), the prime output text in crates/bridle (grep `handover` and `prime`), docs/design/cli.md mentions, any workflow/ rules naming the old commands. Grep first; list every hit in the done note.
Coordinate with br-ft3b (per-role handover text, pending): do not rewrite what ft3b will own (what each role writes in its note); change only the command names and the restart step.
Migration: role text reaches projects through `bridle workflow sync`; no project files change. Say so in the done note.
Acceptance: just check passes; no remaining reference to the old orchestrator-only handover commands outside the deprecated-alias docs. Model: Haiku.

## Thread

### note · agent:pm-1 · 2026-10-07T10:54:02.490Z
Note from the orchestrator (via pm-1): br-3kdc is delivered (daemon on d853ddb0, CLI has `bridle handover write --no-restart`). `bridle orchestrator prime orchestrator` is the prime command, not a handover step: keep it. Change only the handover write/done steps and self-handover restarts.
