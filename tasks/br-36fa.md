+++
id = "br-36fa"
title = "Shutdown must not hang on open event streams; log shutdown requests (zm95, sed3)"
kind = "bug"
state = "planned"
created_at = "2026-09-29T05:21:03.607Z"
updated_at = "2026-09-29T05:21:05.250868Z"
+++

Tickets: docs/questions/open/shutdown-waits-on-open-event-streams-zm95.md and log-shutdown-requests-sed3.md (read both; they are the brief). Graceful shutdown must end open SSE streams (e.g. the TUI's) so the daemon exits promptly; and shutdown requests are logged at warn with the caller principal. Files: crates/bridle-daemon/src/lib.rs, server.rs. Resolve both tickets per docs/README.md. Acceptance: `just check` passes; a test that opens an events stream, requests shutdown, and the daemon exits within a couple of seconds. Model: Sonnet.
