+++
id = "br-e25d"
title = "Project migrations: one command applies pending bridle upgrades to a project, tracked and logged"
kind = "feature"
state = "planned"
created_at = "2026-10-01T19:21:23.829Z"
updated_at = "2026-10-01T19:22:40.151554Z"
+++

Ticket: docs/tickets/open/project-migrations-one-command-applies-pending-bridle-upgrad-xebc.md (read it). First consumer: v3dk (br-2e6e), whose ticket-kind backfill will be the first real migration, in its own task after this one.

Goal: one command applies, to a project, every migration shipped with the running bridle that the project has not had yet; each runs once; the result is auditable by the human and readable by agents; and it is easy to run for all projects.

Shape (KISS; propose differences in the done report, do not silently diverge):
- `bridle migrate [--dry-run] [--project <name> | --all]`, placed where the a67t grouping fits.
- A migration is a Rust fn registered in an ordered const list in the bridle crate: id like "0001-short-name", a description, and run(ctx) -> Result<Report>. Each must be idempotent and must not touch anything outside the project's own bridle files (ticket frontmatter, .bridle/config.toml, docs layout). A migration may refuse, with a clear message, if the files it edits have uncommitted changes.
- Applied state lives IN THE PROJECT, committed with it: `.bridle/migrations.toml` listing applied ids with applied_at (UTC) and the bridle version. The next run applies only ids not listed, in order. A missing file means none applied.
- Audit trail: each run appends, per migration, id, files changed and a summary to `.bridle/migrations.log` (human-readable, append-only) and, if the project's daemon is reachable, records an event so agents can read it (`bridle events`); a daemon being down never fails a migration.
- `--dry-run` prints what each pending migration would change and writes nothing. `--all` iterates the projects in the daemon registry (as `bridle daemons` lists them), one at a time, stopping at the first failure and reporting which were done.
- NEVER auto-run: only when someone invokes it; no hook, no daemon start-up step. Running it on a real project of the human's is the human's decision, on a branch (existing-projects rule).
- Ship ONE no-op sample migration (0000-baseline) so the machinery is exercised; real backfills come in later tasks.
Tests: pending list computed from migrations.toml; a run applies in order and records; a second run changes nothing; --dry-run writes nothing; a failing migration records nothing for itself and stops; --all across two temp projects. Docs: cli.md, a short design note docs/design/migrations.md linked from docs/README.md, CHANGELOG. Acceptance: just check passes. Model: Sonnet.
Out of scope: the ticket-kind backfill (v3dk), the daemon's SQLite migrations (already automatic), rollback/down migrations, running migrations from the daemon.
