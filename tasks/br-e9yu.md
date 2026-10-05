+++
id = "br-e9yu"
title = "Per-project sessions (aide) share one handover file and one identity across projects; key them by project"
kind = "bug"
state = "planned"
created_at = "2026-10-05T00:21:30.006Z"
updated_at = "2026-10-05T00:21:42.642057Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: e9yu
Ticket: docs/tickets/open/per-project-sessions-aide-share-one-handover-file-and-one-id-e9yu.md (read it; it quotes the human).

Approval: the human, via the bridle-ui aide (m-0259, 2026-10-04 ~8:25 PM ET), marked critical: "The iOS file [aide file] is always shared between projects. That's just ridiculously poor planning. File is critical, but fix that".

Goal: a per-project session's handover note is keyed by project as well as identity, so two projects' aides (and advisors) on one machine never share a file.
- crates/bridle-daemon/src/sessions.rs:125 handover_note(identity) builds $BRIDLE_HOME/handover/<identity>.md. Key it by project too (recommended: $BRIDLE_HOME/handover/<project>/<identity>.md). The session already carries its project (SessionInfo.project).
- crates/bridle/src/session.rs:203 and :226 take_handover(&home, identity) on start: read from the same per-project path.
- Migration: on start, if the new path is empty and the old $BRIDLE_HOME/handover/<identity>.md exists, don't guess its project; leave it and say so in the prompt ("an old shared note may be at <path>"), so no project picks up another's note. KISS: no automatic move.
- Then CHECK, and fix if keyed by identity alone, the same collision in: the session registry (sessions.rs register/lookup: does a second project's aide replace or shadow the first's entry?), `bridle session restart aide`, and refuse_if_running("aide", &project). Write what you found per item on the task thread, fixed or already safe, with the line.
- Docs: docs/design/agent-host/orchestrator-supervision.md (lines ~228 and ~244, the path) and any doc naming the old path.
Acceptance: just check green; tests: two sessions with the same identity in different projects get different handover paths, and starting one doesn't pick up the other's note; plus a test for any registry/restart collision you fix.
Model: sonnet.
Out of scope: identity changes beyond the path (seats are gtzx); the h3ar/75h2 waiter work.
