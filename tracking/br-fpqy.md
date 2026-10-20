+++
id = "br-fpqy"
title = "bridle session stop: hand over, then stop an advisor, aide or orchestrator session until it is started again"
kind = "feature"
state = "built"
created_at = "2026-10-20T00:02:27.424Z"
updated_at = "2026-10-20T01:12:51.743429Z"
created_by = "external:advisor/usage"
watchers = [
    "external:advisor/usage",
    "agent:wfpqy",
]
branch = "bridle/wfpqy"
commit = "83c11fee3c21947011d4885259698fc36ba6adb0"
summary = "Added 'bridle session stop <role>[/<name>]' (crates/bridle/src/session.rs, cli.rs): factored restart's handover ask/wait into obtain_handover (reused by both); stop asks for the note (told to use --no-restart), holds the orchestrator first-then-kills (hold after the note), SIGTERM/KILLs the launcher's children by pid, ends the waiter via POST /v1/wake/stop for external:<identity>, prints how to start again (session advisor/aide, or 'bridle orchestrator release'). Refuses under an agent and from inside the session itself. Tests in session_test.rs; docs/design/cli.md, advisor/aide role text, CHANGELOG updated. Caveat: restart_hold_test cancel_ends_an_upgrade_drain_quietly flaked once in a full run (unrelated), passed alone and on re-run of just check (1905 passed)."
ticket = "fpqy"
+++

docs/tickets/open/bridle-session-stop-hand-over-then-stop-an-advisor-aide-or-o-fpqy.md

## Thread

### note · external:advisor/usage · 2026-10-20T00:02:36.645Z
From advisor (usage): readied. The human asked for it directly (quoted in the ask).

### note · agent:wfpqy · 2026-10-20T01:12:51.186Z
built: 83c11fee3c21947011d4885259698fc36ba6adb0 (branch bridle/wfpqy)

### note · agent:wfpqy · 2026-10-20T01:12:51.743Z
done: bridle session stop; just check exit 0, 1905 tests; 83c11fee3c21947011d4885259698fc36ba6adb0
