+++
id = "br-kae5"
title = "bridle docs: a CLI overview of how bridle works, for agents in any project"
kind = "feature"
state = "planned"
created_at = "2026-10-04T13:34:26.958Z"
updated_at = "2026-10-04T13:40:39.181888Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: kae5
Feature. Ticket: docs/tickets/open/bridle-docs-a-cli-overview-of-how-bridle-works-for-agents-in-kae5.md. The human, 2026-10-04 (via the NUC orchestrator, m-0338): 'approve m7mp and kae5 afterwards' (after br-fc9a); the ask: agents in any project learn how bridle works from the CLI, without the bridle repo; a man page later, not now.
Build, smallest first: 'bridle docs' lists topics; 'bridle docs <topic>' prints a short overview. Topics (propose a minimal set, about 6-8): overview, roles, priming-and-rules, sessions, tasks-and-tickets, queue, messages, workflow-layers. Source: short markdown files kept in the repo (e.g. docs/cli/<topic>.md, written for an agent in another project, each under about 100 lines, linking to the real design doc by path for more) embedded in the binary with include_str! so they work anywhere; derive the content from the existing design docs (docs/design/*.md, agent-host/*), don't invent behaviour, and say 'planned' where the design says so. A test that every listed topic has a non-empty embedded file and every embedded file is listed. Files: crates/bridle/src (new commands/docs.rs, cli.rs registration), the new markdown, docs/design/cli.md, docs/README.md, CHANGELOG. Acceptance: just check passes; 'bridle docs' and 'bridle docs roles' print sensibly. Model: Sonnet. Migration: none. Out of scope: a man page, project-specific docs, editing existing projects.

## Thread

### note · external:orchestrator · 2026-10-04T13:39:53.800Z
The human, 2026-10-04: 'approve m7mp and kae5 afterwards' (after br-fc9a).
