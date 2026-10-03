+++
id = "br-36fa"
title = "Shutdown must not hang on open event streams; log shutdown requests (zm95, sed3)"
kind = "bug"
state = "integrated"
created_at = "2026-09-29T05:21:03.607Z"
updated_at = "2026-09-29T05:29:00.038984Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
branch = "bridle/shutdown-hang"
commit = "62a97d6"
summary = "The zm95 fix (SSE streams end on shutdown, 5s bounded HTTP drain) and its test were already on main (681d0c3); this task adds WARN logs for shutdown requests (POST /v1/shutdown with principal id; SIGINT/SIGTERM) in server.rs and lib.rs, a CHANGELOG line, and moves tickets zm95 and sed3 to resolved/."
+++

Tickets: docs/questions/open/shutdown-waits-on-open-event-streams-zm95.md and log-shutdown-requests-sed3.md (read both; they are the brief). Graceful shutdown must end open SSE streams (e.g. the TUI's) so the daemon exits promptly; and shutdown requests are logged at warn with the caller principal. Files: crates/bridle-daemon/src/lib.rs, server.rs. Resolve both tickets per docs/README.md. Acceptance: `just check` passes; a test that opens an events stream, requests shutdown, and the daemon exits within a couple of seconds. Model: Sonnet.

## Thread

### note · agent:manager-2 · 2026-09-29T05:29:00.038Z
integrated: 62a97d6 (branch bridle/shutdown-hang)
