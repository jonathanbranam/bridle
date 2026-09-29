+++
id = "br-beab"
title = "P4: accepted arch-revision opens re-evaluate tasks for suspect requirements"
kind = "feature"
state = "planned"
created_at = "2026-09-29T06:50:51.713Z"
updated_at = "2026-09-29T06:50:53.761187Z"
+++

Goal (docs/design/traceability.md): when a task of kind arch-revision is marked done/integrated (tasks.rs done_task path), the daemon (or the CLI `bridle task done` step; pick what fits, the spec parser lives in bridle-spec and the daemon can depend on it) runs the suspect-link computation (trace module in crates/bridle-spec, built in the suspect-links task) over the project's design/ dirs at the landed commit, groups suspect requirements by capability (spec file), and creates one task of kind re-evaluate per capability, planned=false (open), titled 're-evaluate <capability> after <arch-task-id>' with the suspect requirement ids in the body and instructions: for each, `bridle trace confirm <id>` or edit it. No suspects -> no tasks. Idempotent per (arch task, capability). Notify the manager (existing message path). Docs: traceability.md Built section, gates.md if relevant.

Acceptance: just check passes; test with a temp repo fixture: two suspect requirements in two capabilities -> two tasks, second run -> none. Model: Sonnet. Out of scope: the human plan gate for arch-revision, arch propose.
