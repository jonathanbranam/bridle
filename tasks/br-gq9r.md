+++
id = "br-gq9r"
title = "Interactive sessions hand themselves over and restart at 200k instead of only being told to"
kind = "feature"
state = "planned"
created_at = "2026-10-05T09:59:54.280Z"
updated_at = "2026-10-05T12:34:21.913949Z"
created_by = "external:aide"
watchers = ["external:aide"]
summary = "The 200k and 250k session context step notes (crates/bridle-daemon/src/sessions.rs) now tell the session to restart itself: say 'I'm at 200k; restarting', run 'bridle session restart <identity>', write the handover it asks for; the human can still 'session keep' and nothing is forced before 300k. aide.md and advisor.md say the same; orchestrator-supervision.md, CHANGELOG updated; new test the_200k_step_tells_the_session_to_hand_over_and_restart_itself. Role prompts reach existing projects with no per-project change: a role with no system_prompt in .bridle/config.toml reads workflow/base/roles/<role>.md when the session launches (docs/design/workflow-layers.md), so they pick it up at the next launch, unless a project sets its own system_prompt. Caveat: restart without --fresh asks the session for a note itself, so the step text says 'write the note it asks for' rather than writing one first."
+++

original id: gq9r
Ticket (the ask, with the human's words; read first): docs/tickets/open/interactive-sessions-hand-themselves-over-and-restart-at-200-gq9r.md
Depends on br-4s3z (a session can restart itself); start only after it merges.

Goal: interactive sessions (aide, advisor) act on the 200k context step themselves instead of waiting for the human: on the 200k step the session says "I'm at 200k; restarting", writes its handover (daemon record, per identity and project), and restarts itself with the 4s3z mechanism. The human can override (`session keep`) or choose a fresh restart; nothing is forced before the 300k hard limit. The orchestrator keeps its own numbers.
Change: the step messages in the daemon (see docs/design/agent-host/orchestrator-supervision.md, "Context steps"; grep the 150k/200k/250k step text in crates/bridle-daemon) and workflow/base/roles/aide.md and advisor.md, so the wording is "restart yourself", with authority to do it. Update the design doc. Check how role-prompt changes reach existing projects (workflow sync) and say so in the done note; no per-project file changes should be needed.
Acceptance: just check passes; a test that the 200k step text tells the session to hand over and restart; docs updated.
Model: Sonnet. Out of scope: orchestrator context steps, forcing restarts below 300k, the 4s3z mechanism itself.

## Thread

### note · agent:self-restart · 2026-10-05T12:34:19.640Z
done: sessions restart themselves at 200k; just check exit 0, 1227 tests passed; checked on 3b-less commit, see below

### note · agent:self-restart · 2026-10-05T12:34:21.913Z
correction: the green just check (exit 0, 1227 passed) ran on the commit before the merge; the tip is merge 8e04b6b5, and main brought in only a ticket doc, so no re-run. Pre-merge commit: de6f04a4
