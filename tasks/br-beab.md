+++
id = "br-beab"
title = "P4: accepted arch-revision opens re-evaluate tasks for suspect requirements"
kind = "feature"
state = "planned"
created_at = "2026-09-29T06:50:51.713Z"
updated_at = "2026-09-29T07:44:41.089066Z"
summary = """Landing an arch-revision task (server.rs done_task -> open_reevaluate_tasks) now computes suspect links over design/{goals,architecture,specs} in the repo checkout (new bridle-daemon/src/reevaluate.rs, using bridle_spec::trace::Graph), groups suspect requirement ids by spec file stem, and opens one open (unplanned) re-evaluate task per capability titled "re-evaluate <cap> after <arch-id>" with the ids and confirm-or-edit instructions; existing same-title tasks are skipped (idempotent); the manager (else human) gets a note. Unparseable trace inputs are logged and skipped. Test: tests/reevaluate_test.rs (two capabilities -> two tasks; a feature landing -> none). Docs: traceability.md Built section; CHANGELOG. Caveat: the acceptance "second run -> none" is covered by the title guard but not tested, since done can't be re-run on the same task. Full `just check` has one failure, ports_test::alloc_skips_reserved_taken_and_listening_ports_and_release_frees (from main's br-57be, AddrInUse at test line 45; likely the test daemon's ephemeral listener taking base+1); it fails deterministically here and does not touch this change."""
+++

Goal (docs/design/traceability.md): when a task of kind arch-revision is marked done/integrated (tasks.rs done_task path), the daemon (or the CLI `bridle task done` step; pick what fits, the spec parser lives in bridle-spec and the daemon can depend on it) runs the suspect-link computation (trace module in crates/bridle-spec, built in the suspect-links task) over the project's design/ dirs at the landed commit, groups suspect requirements by capability (spec file), and creates one task of kind re-evaluate per capability, planned=false (open), titled 're-evaluate <capability> after <arch-task-id>' with the suspect requirement ids in the body and instructions: for each, `bridle trace confirm <id>` or edit it. No suspects -> no tasks. Idempotent per (arch task, capability). Notify the manager (existing message path). Docs: traceability.md Built section, gates.md if relevant.

Acceptance: just check passes; test with a temp repo fixture: two suspect requirements in two capabilities -> two tasks, second run -> none. Model: Sonnet. Out of scope: the human plan gate for arch-revision, arch propose.
