+++
id = "br-krz8"
title = "bridle session refuses a second session of an identity that's already running"
kind = "bug"
state = "planned"
created_at = "2026-10-04T18:39:04.359Z"
updated_at = "2026-10-04T18:39:41.250989Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: krz8
Build docs/tickets/open/bridle-session-refuses-a-second-session-of-an-identity-that-krz8.md (read it). bridle session <role> refuses to start when a session of the same identity in the same project is registered and its process is alive (bridle status sessions, the pid). The message names the running one (pid, pane, machine) and says how to replace it (bridle session restart <identity>). A registered session whose process is gone does not block. Test with a stub claude (BRIDLE_LAUNCHER_TEST=1). Docs: cli.md bridle session. Touches crates/bridle/src/session.rs: starts after br-3397 merges (dependency edge). Acceptance: just check passes. Model: Sonnet. Out of scope: anything else in session.

## Thread

### note · external:orchestrator · 2026-10-04T18:39:29.274Z
The human, 2026-10-04, via aide (m-4738), choosing option 2 on br-3397 (land as built, file follow-ups): "Agree with 2 - we need to start using specs first bridle".
