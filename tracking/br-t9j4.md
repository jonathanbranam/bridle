+++
id = "br-t9j4"
title = "7bs2 gaps: just install signs a copy and renames; the gateway checks its own tailnet listener"
kind = "fix"
state = "claimed"
created_at = "2026-10-16T10:03:28.515Z"
updated_at = "2026-10-16T21:17:47.147652Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:t9j4",
    "agent:t9j4b",
]
branch = "bridle/t9j4"
commit = "0454f3b8bc9bc2f2e61976a6de98929741c8b6c5"
summary = "Ticket br-t9j4: just install now builds into a private cargo root and runs new 'bridle sign install SRC DEST' (signing::install: signed copy named bridle, renamed into place). The gateway (macOS, non-loopback bind) probes its own listener after start (crates/bridle-gateway/src/reachable.rs) and messages each local project's orchestrator naming the firewall; unit test uses a closed port. cli.md and CHANGELOG updated. just check green: 1756 passed. Caveat: the justfile install recipe was not run (it would overwrite the live binary)."
ticket = "t9j4"
+++

docs/tickets/open/7bs2-gaps-just-install-signs-a-copy-and-renames-the-gateway-t9j4.md

## Thread

### note · agent:t9j4 · 2026-10-16T10:44:03.419Z
built: a771c61e (branch bridle/t9j4)

### note · agent:t9j4 · 2026-10-16T10:44:03.519Z
done: just install via sign install; gateway self-probe + incident; just check exit 0, 1756 tests; a771c61e

### note · agent:manager-2 · 2026-10-16T11:08:06.939Z
integrated: 0454f3b8bc9bc2f2e61976a6de98929741c8b6c5 (branch bridle/t9j4)

### note · agent:manager-2 · 2026-10-16T11:08:30.421Z
cleanup: removed agent t9j4, branch bridle/t9j4

### note · external:advisor/product-manager · 2026-10-16T21:08:26.424Z
advisor (product-manager), verifying the landing (0454f3b8): item (a) met. Gap in (b): the ask said 'the same incident' as the daemon's; the gateway's tell_orchestrators only sends each project's orchestrator a note. No orchestrator.incident event is recorded and nothing reaches the human. Reopened for that remainder.

### note · agent:pm-1 · 2026-10-16T21:08:35.506Z
Brief for the remainder (pm-1): item (a) is done. For (b), when the gateway's self-probe fails it must raise the same incident the daemon does: record an orchestrator.incident event (reuse the daemon's code path or helper, do not copy it) so it reaches the human, not only a note to each project's orchestrator (tell_orchestrators in the gateway). Keep the note too. Test with a fake unreachable listener that the incident event is recorded. Acceptance: just check. Model: Sonnet. Out of scope: disabling the firewall.
