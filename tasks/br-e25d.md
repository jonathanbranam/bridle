+++
id = "br-e25d"
title = "Project migrations: one command applies pending bridle upgrades to a project, tracked and logged"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T19:21:23.829Z"
updated_at = "2026-10-01T19:32:40.557112Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/migrate"
commit = "f258a4a46a2272e6e6ec82d199d3f7c6b987487b"
summary = "Added `bridle migrate [--dry-run] [--project NAME | --all]` (crates/bridle/src/migrate.rs, handler in commands/mod.rs). Migrations are fns in an ordered const MIGRATIONS list; shipped one no-op 0000-baseline. Applied state is `.bridle/migrations.toml` in the project, with an append-only `.bridle/migrations.log`; each applied migration also becomes a `project.migrated` event via a new `POST /v1/migrations` (types.rs MigrationRecord, client, server), best effort. Never auto-run. Differences from brief: (1) `migrate` is top-level, not in a group; `--project` is the existing global flag. (2) The brief said events if the daemon is reachable but no event-posting endpoint existed, so I added that small one. (3) --all covers registry (running) daemons only. (4) --all across two projects is covered by the per-project apply tests, not an end-to-end CLI test. Docs: cli.md, design/migrations.md, README index, CHANGELOG."
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

## Thread

### note · agent:migrate · 2026-10-01T19:32:20.066Z
done: bridle migrate (0000-baseline, migrations.toml/log, project.migrated event via new POST /v1/migrations); just check exit 0, 962 tests; differences listed in the task summary; b3dcbcb

### note · agent:manager-2 · 2026-10-01T19:32:31.579Z
integrated: f258a4a46a2272e6e6ec82d199d3f7c6b987487b (branch bridle/migrate)

### note · agent:manager-2 · 2026-10-01T19:32:40.557Z
cleanup: removed agent migrate, branch bridle/migrate
