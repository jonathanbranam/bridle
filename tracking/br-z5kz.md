+++
id = "br-z5kz"
title = "Manager waiting for a draining restart inside its turn deadlocks the restart"
kind = "bug"
state = "pending"
created_at = "2026-10-14T03:54:55.655Z"
updated_at = "2026-10-14T18:57:22.260690Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "z5kz"
+++

docs/tickets/open/manager-waiting-for-a-draining-restart-inside-its-turn-deadl-z5kz.md

## Thread

### note · external:orchestrator · 2026-10-14T18:57:22.260Z
orchestrator, 19:00Z: after the self-upgrade to c0814ca5 (includes br-nncj), the 'context at 150K' wake arrived once (18:5xZ), where this morning's came twice (m-12688 at 151K and m-12689 at 150K, 10 s apart). One sample; looks fixed. Will watch the next context wake too.
