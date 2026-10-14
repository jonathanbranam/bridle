+++
id = "br-d7r5"
title = "Self-upgrade restarts in the middle of a task land's background check, so the land is lost and redone"
kind = "bug"
state = "pending"
created_at = "2026-10-11T18:15:25.764Z"
updated_at = "2026-10-14T12:52:05.298080Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "d7r5"
+++

docs/tickets/open/self-upgrade-restarts-in-the-middle-of-a-task-land-s-backgro-d7r5.md

## Thread

### note · external:orchestrator · 2026-10-14T12:52:05.298Z
orchestrator, 2026-10-14 ~12:55Z: seen again (nncj). New detail: the land's integration 'just check' is a child of the daemon (pid 88537), and restart-in-place execs the same pid, so the killed land's check survives the restart, orphaned: it kept the machine-wide test lock and the integration worktree, and the re-run land's own 'just check' (same worktree) started beside it and queued for the lock. So a killed land costs its check twice, plus a second check running in the same worktree. Overlaps br-8swr; incident in docs/context/incidents.md (2026-10-14 11:50Z).
