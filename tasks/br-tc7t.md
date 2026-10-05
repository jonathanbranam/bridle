+++
id = "br-tc7t"
title = "A landing that needs a UI install or gateway restart to show says so, and someone does it"
kind = "bug"
state = "integrated"
created_at = "2026-10-04T21:27:39.835Z"
updated_at = "2026-10-05T07:42:17.856075Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
priority = "high"
branch = "bridle/landing-install"
commit = "b80f42787ad191bdb575017783aab43f26ad32ae"
summary = "br-bek3 had already delivered the gateway restart (self-watch re-exec when the bridle binary file changes; test a_replaced_binary_is_re_executed replaces the binary like cargo install). Task shrank to the docs gap: daemon.md 'Automatic upgrade' now says the gateway follows an upgrade by itself, no daemon hook. No code change, no CHANGELOG line (bek3's covers it)."
+++

original id: tc7t
Bridle-repo half of docs/tickets/open/a-landing-that-needs-a-ui-install-or-gateway-restart-to-show-tc7t.md (read it). The human chose full automation (option C). This task: the gateway restarts itself on a bridle upgrade, like the daemon (daemon.md 'Automatic upgrade': quiet point, in-place restart). It builds on br-bek3 (detached gateway plus its restart-on-upgrade spec), so if br-bek3 already delivered the restart, this task shrinks to verifying it end to end with a test and closing the gap in the docs; say so on the thread and finish. Files: crates/bridle-daemon (upgrade path), crates/bridle-gateway, daemon.md, human-web-ui.md. Acceptance: just check passes; a test that an upgrade restart also restarts a running gateway. Model: Sonnet. Out of scope: installing the UI into ~/.bridle/ui (the 'bridle-ui daemon builds green main like self_upgrade' half): that belongs to the bridle-ui project, not this repo; the orchestrator is raising it there.

## Thread

### note · external:orchestrator · 2026-10-04T21:27:43.699Z
Approval: the human, 2026-10-04 via aide (m-0140): "I want option C. All of this should be automated." Scope per aide: (1) the bridle-ui daemon installs the UI like self_upgrade, building green main into ~/.bridle/ui, no role involved; (2) the gateway restarts itself on a bridle upgrade like the daemon. Full quote in the ticket (928543b3).

### note · agent:pm-1 · 2026-10-04T21:28:07.609Z
priority: normal -> high

### note · external:aide · 2026-10-04T21:42:32.974Z
watching the task

### note · agent:landing-install · 2026-10-05T07:29:49.707Z
done: gateway restart already delivered by br-bek3 (self-watch re-exec, existing test replaces binary like cargo install); added daemon.md paragraph, docs only. just check exit 0, 1204 tests passed, sha d024f092. An earlier run hit 5 load timeouts in daemon spawn/renew tests; they pass alone and on rerun.

### note · agent:manager-2 · 2026-10-05T07:37:31.151Z
integrated: b80f42787ad191bdb575017783aab43f26ad32ae (branch bridle/landing-install)

### note · agent:manager-2 · 2026-10-05T07:42:17.856Z
cleanup: removed agent landing-install, branch bridle/landing-install
