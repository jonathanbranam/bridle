+++
id = "br-4s3z"
title = "bridle session restart says it restarted a session that's still running, and types the relaunch into it"
kind = "bug"
state = "planned"
created_at = "2026-10-05T00:33:50.973Z"
updated_at = "2026-10-05T00:34:27.100746Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: 4s3z
Build docs/tickets/open/bridle-session-restart-says-it-restarted-a-session-that-s-st-4s3z.md (read it; the Fix section is the spec). In crates/bridle/src/session.rs, stop_session (and the restart path): (1) confirm the old launcher pid is actually gone before typing the relaunch into the pane; if SIGTERM plus the 20 s wait did not do it, escalate (SIGKILL to that exact pid's child, never by name: rule no-kill-by-name) or fail with a clear error, and never type into a live session; (2) a session can restart itself: run the stop-and-relaunch detached from the session it kills (or have the daemon do it) so killing the caller does not kill the restart; (3) print 'restarted' only after the new session has registered (a new pid in bridle status), else report failure. Test with the stub claude (BRIDLE_LAUNCHER_TEST=1): restart stops the old pid and registers a new one; an unkillable stub makes restart fail without typing the relaunch; self-restart completes. Update cli.md bridle session restart. Overlap: br-e9yu/br-cyvf change the handover step that restart waits on (~/.bridle/handover); check their state and run after them if they touch the restart path. Acceptance: just check passes. Model: Sonnet. Out of scope: the handover record itself, other session commands.
