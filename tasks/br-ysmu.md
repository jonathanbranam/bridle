+++
id = "br-ysmu"
title = "CI watch missed 13 red runs on main; first ci_failed wake came 40 minutes late"
kind = "bug"
state = "pending"
created_at = "2026-10-05T03:21:20.785Z"
updated_at = "2026-10-05T03:21:29.947048Z"
created_by = "external:orchestrator@nuc"
watchers = ["external:orchestrator@nuc"]
+++

original id: ysmu
docs/tickets/open/ci-watch-missed-13-red-runs-on-main-first-ci-failed-wake-cam-ysmu.md

submitted by external:orchestrator@nuc

meta-notes-ui ([ci] github = true, daemon on 7405, project added 2026-10-05). CI on main failed from 97e0b13 (02:41 UTC) through df5f90d: 13 red pushes, 9 of them merges. The daemon recorded a single ci.completed event (df5f90d at 03:20:10) and the orchestrator's first ci_failed wake came then. Meanwhile the manager kept merging; its allowlist denies gh, so it couldn't see CI itself and relied on the wake. Expected: a ci.completed event per run on main, and a wake on the first failure. Cause unknown (new project's watcher start? polling only the newest run?). Also consider: give the manager a bridle command to read the latest CI result on main, so 'never merge on red' doesn't need gh.

## Thread

### note · external:orchestrator@nuc · 2026-10-05T03:21:20.786Z
submitted by external:orchestrator@nuc

### note · agent:pm-1 · 2026-10-05T03:21:29.947Z
Triage (pm-1): accept, high value: merges went in on red on an onboarded project. Ticket minted (uncommitted; needs committing on main). Stays pending: approve with `bridle task ready br-ysmu` and I plan it (Sonnet, bridle-daemon CI watcher; the manager's read command may split off as a small second task).
