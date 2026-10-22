---
id: br-6gdp
title: "A migration run leaves the project clone dirty: commit what it writes"
kind: bug
created: 2026-10-22
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Part 1 of 3 of the orchestrator's finding (submitted as br-6gdp; the human, 2026-10-21: "Why do we have files which aren't committed? that is a red flag that something is being lost somewhere"). Siblings: ticket-file commits and scratch ignore, filed as separate tickets (related).

A migration run leaves the project clone dirty. Seen on dalek-home: `bridle` 0.7.5 applied 0000, 0006 and 0007 on 2026-10-17 and wrote `.gitignore`, `.bridle/migrations.toml` and `.bridle/migrations.log` with no commit; four days uncommitted. Other projects that ran migrations likely have the same.

Fix: one commit per migration run, covering exactly the files the run wrote (on the project's current branch; for an existing project on a trial branch, that branch, never main/dev: rule `existing-projects`). If the run cannot commit (not a git clone, a dirty index of the user's own changes), it prints that and leaves them, never sweeping the user's unrelated changes into the commit.

Migration plan for existing projects: those already dirty from past runs (dalek-home) get the leftover files committed by the next run of `bridle project migrate` (or a one-off note to their orchestrator); say which in the ticket's summary.

Acceptance: running a migration leaves the clone clean (test with a temp git repo); unrelated user changes stay out of the commit. `just check`. Docs: docs/design/cli.md or the migrations doc. Model: Sonnet. Files: crates/bridle-daemon migrate code (grep migrations.log). Related: br-df8w (same migrate files; run after it if it is still open).
