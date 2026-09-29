+++
id = "br-cfe8"
title = "Product brief: how bridle specs work today (8awb, part 1)"
kind = "chore"
state = "planned"
created_at = "2026-09-29T12:28:21.816Z"
updated_at = "2026-09-29T12:28:23.969815Z"
size = "S"
+++

Ticket: docs/questions/open/*8awb.md (read the human's ask). Goal: ONE short brief, docs/briefs/specs.md (plus docs/briefs/README.md as the index, linked from docs/README.md), that the human reads to decide whether to migrate a project (meta-notes) onto bridle specs. Plain language, one to two pages, no jargon left unexplained. Cover: what specs are for; the whole flow end to end (a spec file, ids, bridle spec check/id/export/coverage/import openspec, test adapters for pytest and vitest, traceability links, impact/conflicts, how a task uses them); what is built vs not yet, verified against the code and CHANGELOG.md, not the design docs (docs/design/specs.md, spec-flow.md, specs-to-tests.md, traceability.md, impact-and-conflicts.md are design and may run ahead; crates/bridle-spec, crates/bridle/src/spec_*.rs, docs/design/cli.md are as built); what migrating a project involves, what it costs the project, what the human would review (the trial-branch rule, workflow/base/rules/existing-projects.md) and what could go wrong. Mark every claim built, partly built or planned. End with a short 'decisions for you' list. Docs only: do not edit code or design docs; if you find a design doc contradicting the code, note it in the brief's last section rather than fixing it. Acceptance: just check passes; every command named in the brief exists in bridle --help. Model: Sonnet. Out of scope: briefs for the task system and other parts (later tasks), publishing as an Artifact.
