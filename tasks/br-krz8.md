+++
id = "br-krz8"
title = "bridle session refuses a second session of an identity that's already running"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T18:39:04.359Z"
updated_at = "2026-10-04T20:42:11.478608Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/dup-session"
commit = "3734cbc5c7bd23f5f7b839a482dcb1ff243b41b3"
summary = "bridle session advisor|aide now refuses to start when the same identity is registered in the project with a live process (ps finds the pid). Message names pid, pane, machine and says `bridle session restart <identity>`. A registered session whose process is gone, one on another machine, or an unreachable daemon does not block. Code: refuse_if_running in crates/bridle/src/session.rs. Orchestrator is not covered (it does not register, only keeps a pid file). Tests in session_test.rs use a fake HTTP daemon via --url. Gotcha: tests inherit BRIDLE_URL/BRIDLE_TOKEN from an agent's env and would hit the real daemon, so session_test.rs and tools_only_test.rs now remove them. Docs: cli.md, CHANGELOG."
+++

original id: krz8
Build docs/tickets/open/bridle-session-refuses-a-second-session-of-an-identity-that-krz8.md (read it). bridle session <role> refuses to start when a session of the same identity in the same project is registered and its process is alive (bridle status sessions, the pid). The message names the running one (pid, pane, machine) and says how to replace it (bridle session restart <identity>). A registered session whose process is gone does not block. Test with a stub claude (BRIDLE_LAUNCHER_TEST=1). Docs: cli.md bridle session. Touches crates/bridle/src/session.rs: starts after br-3397 merges (dependency edge). Acceptance: just check passes. Model: Sonnet. Out of scope: anything else in session.

## Thread

### note · external:orchestrator · 2026-10-04T18:39:29.274Z
The human, 2026-10-04, via aide (m-4738), choosing option 2 on br-3397 (land as built, file follow-ups): "Agree with 2 - we need to start using specs first bridle".

### note · agent:dup-session · 2026-10-04T20:34:39.063Z
done: bridle session advisor|aide refuses a second live session of an identity; just check green (1177 tests); 3bc99449

### note · agent:dup-session · 2026-10-04T20:37:00.342Z
main merged, just check green (1177 tests); 3052e6f6

### note · agent:manager-2 · 2026-10-04T20:37:04.929Z
integrated: 3734cbc5c7bd23f5f7b839a482dcb1ff243b41b3 (branch bridle/dup-session)

### note · agent:manager-2 · 2026-10-04T20:42:11.478Z
cleanup: removed agent dup-session, branch bridle/dup-session
