+++
id = "br-x56y"
title = "bridle send from any agent fails since the 3haz build: a daemon found by BRIDLE_URL has no project, so BRIDLE_PROJECT reads as another project"
kind = "bug"
state = "planned"
created_at = "2026-10-06T23:14:33.594Z"
updated_at = "2026-10-06T23:15:17.965102Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: x56y
CRITICAL: no agent can `bridle send` on any daemon since the br-3haz build; takes the next free worker slot. Ticket (cause, repro, errors; read all of it): docs/tickets/open/bridle-send-from-any-agent-fails-since-the-3haz-build-a-daem-x56y.md . Related: incident br-2ax5 (same release).
Cause: every agent runs with BRIDLE_PROJECT=<its project> and BRIDLE_URL. `own_daemon_for_other_project` (crates/bridle/src/commands/misc.rs) resolves the own daemon from BRIDLE_URL; a daemon found by URL has `project == None`, so `own.project == Some(project)` is false and the send is routed to the own daemon's outbox as cross-project mail, which the daemon refuses (crates/bridle-daemon/src/server.rs, "that is this daemon's own project"; with --task: "--task isn't supported for another project's daemon yet").
EXACT FIX (decided):
1. When the own daemon was found by URL (project None), learn its project from the daemon itself (use what its status/health API already returns; check crates/bridle-api for a project name field; if none exists, add an additive optional field to the status response, updating daemon, CLI, TUI and gateway together) and compare that with the requested project. If the daemon cannot say, treat the project as the SAME (send directly): never send to the outbox on an unknown own project.
2. A send is cross-project (outbox, peer token) ONLY when the target project is known to differ from the own daemon's project.
3. Audit every other caller of `own_daemon_for_other_project` and every other `own.project == Some(...)` style comparison in crates/bridle for the same mistake (task watch, ticket submit, handover, etc.) and fix them the same way; list what you found in the done note.
Workaround in use meanwhile (do not remove or document as the fix): `env -u BRIDLE_PROJECT bridle send ...`.
Tests (the gap, as in 2ax5: nothing ran `bridle send` with an agent's real environment): CLI integration tests against a test daemon with BRIDLE_URL and BRIDLE_PROJECT both set to the daemon's own project: `bridle send <agent> "x"` sends directly, and `--task <id>` works too; with BRIDLE_PROJECT naming a DIFFERENT project it still goes through the outbox path; with BRIDLE_URL set and no BRIDLE_PROJECT it sends directly. Use the existing test harness for CLI-against-daemon; no real claude.
Docs: docs/design/cli.md and agent-host docs where "send" and cross-project routing are described, one line on how the own project is determined; CHANGELOG entry (read with a limit).
Acceptance: just check passes; the tests above; manually the repro `BRIDLE_URL=<test daemon> BRIDLE_PROJECT=<its project> bridle send ...` succeeds (against a TEST daemon, never the live one).
Migration: none (CLI behaviour). Model: Sonnet. Out of scope: the 3haz slices 2-4, peer-token minting (br-y7ht), the outbox design.

## Thread

### note · external:orchestrator · 2026-10-06T23:14:33.796Z
From orchestrator: br-x56y is CRITICAL and takes the next free worker slot. No agent can bridle send on any daemon. The brief, cause and repro are on the ticket. Sonnet. Plan it yourself if pm-1 hasn't (it's ready); tell pm-1.
