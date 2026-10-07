+++
id = "br-3kdc"
title = "One handover command, the same for every agent"
kind = "feature"
state = "planned"
created_at = "2026-10-07T10:15:45.545Z"
updated_at = "2026-10-07T10:16:40.623511Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "3kdc"
+++

Ticket: docs/tickets/open/one-handover-command-the-same-for-every-agent-3kdc.md (read it, with the human's verbatim ask). Slice 1 of 2: the command. Slice 2 (role files) is a separate task blocked on this one.

Goal: ONE handover command, identical for every agent (orchestrator, aide, advisors, managers, workers). Today `bridle handover write` records a note for anyone, but `bridle handover done` (POST /v1/orchestrator/handover, only human and external:orchestrator) is the orchestrator-only "state written, restart me" signal, and other interactive sessions go through `bridle session restart --handover`.

Build:
1. `bridle handover write --file <path>|-` stays THE command. After recording the note it also signals "state written, restart me" for the caller's own session, the same way for everyone: orchestrator -> the daemon stops and relaunches it (the existing handover-done path, now reachable by any principal for ITS OWN session only); an interactive advisor/aide session -> the same self-restart `bridle session restart` does today (detached, by pid, see docs/design/cli.md section on `session restart`); a worker -> records the note and prints it was recorded (the daemon already resumes workers; no restart signal). Add `--no-restart` for note-only (a note written mid-session, no relaunch). Read crates/bridle/src/ (handover subcommand), crates/bridle-daemon (orchestrator handover route, session restart code) and docs/design/agent-host/orchestrator-supervision.md sections 6 and 7 first; reuse the existing restart code, do not write a second restart mechanism.
2. The orchestrator-only gate goes away: any principal may signal restart of its OWN session; nobody may restart another's through this command.
3. Compatibility: `bridle handover done` and `bridle orchestrator handover done` keep working as deprecated aliases (hidden from help, one-line stderr note pointing at `bridle handover write`) so running orchestrators and old role text do not break during the upgrade. Removing them is a later task.
4. Fix the help text: it must not say "the orchestrator's handover note"; say every agent's.
5. Update docs/design/cli.md (handover paragraph), orchestrator-supervision.md sections 6/7, and docs/design/agent-host/ API docs for any route change. Wire-format change means bridle-api/src/types.rs, daemon and clients together.
Do NOT edit workflow/base/roles/*.md or the prime text: that is slice 2, after this lands and is delivered. Coordinate with br-ft3b (per-role handover TEXT, pending): this task is mechanics only, do not wait on it.
Migration: no project files change; the command reaches agents on daemon upgrade, role text later via workflow sync (slice 2).
Acceptance: just check passes; tests: write by orchestrator relaunches, write by an advisor triggers self-restart (fake pane/launcher as the existing session-restart tests do), write by a worker only records, --no-restart records only, aliases still work and print the deprecation note.
Model: Sonnet.
