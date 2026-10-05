+++
id = "br-gq9r"
title = "Interactive sessions hand themselves over and restart at 200k instead of only being told to"
kind = "feature"
state = "planned"
created_at = "2026-10-05T09:59:54.280Z"
updated_at = "2026-10-05T10:00:28.349005Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: gq9r
Ticket (the ask, with the human's words; read first): docs/tickets/open/interactive-sessions-hand-themselves-over-and-restart-at-200-gq9r.md
Depends on br-4s3z (a session can restart itself); start only after it merges.

Goal: interactive sessions (aide, advisor) act on the 200k context step themselves instead of waiting for the human: on the 200k step the session says "I'm at 200k; restarting", writes its handover (daemon record, per identity and project), and restarts itself with the 4s3z mechanism. The human can override (`session keep`) or choose a fresh restart; nothing is forced before the 300k hard limit. The orchestrator keeps its own numbers.
Change: the step messages in the daemon (see docs/design/agent-host/orchestrator-supervision.md, "Context steps"; grep the 150k/200k/250k step text in crates/bridle-daemon) and workflow/base/roles/aide.md and advisor.md, so the wording is "restart yourself", with authority to do it. Update the design doc. Check how role-prompt changes reach existing projects (workflow sync) and say so in the done note; no per-project file changes should be needed.
Acceptance: just check passes; a test that the 200k step text tells the session to hand over and restart; docs updated.
Model: Sonnet. Out of scope: orchestrator context steps, forcing restarts below 300k, the 4s3z mechanism itself.
