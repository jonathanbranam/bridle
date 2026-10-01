# Project migrations

Some bridle changes alter what a project keeps in its own repo for bridle: ticket frontmatter,
`.bridle/config.toml`, the docs layout. `bridle migrate` brings a project up to date with the
running binary (ticket xebc). The daemon's SQLite database is separate: it migrates itself
([[docs/design/storage|storage]]).

## Rules

- **Never automatic.** Only `bridle migrate` runs migrations: no hook, no daemon start-up step.
  Running it on one of the human's real projects is the human's call, on a branch.
- A migration is a Rust fn in the ordered `MIGRATIONS` list (`crates/bridle/src/migrate.rs`):
  an id `NNNN-short-name`, a description, `run(ctx) -> Result<Report>`. Ids are append-only.
- It must be **idempotent**, touch only the project's own bridle files, and under `ctx.dry_run`
  write nothing but still report what it would change. It may refuse, with a clear message, when
  the files it edits have uncommitted changes.
- Down migrations don't exist.

## State and audit

- `.bridle/migrations.toml`, committed with the project: `[[applied]]` entries with `id`,
  `applied_at` (UTC) and `bridle_version`. A missing file means none applied. A run applies the
  ids not listed, in list order, recording each as it succeeds.
- `.bridle/migrations.log`: one human-readable line per applied migration (time, id, version,
  summary, files changed); append-only.
- If the project's daemon answers, each applied migration is also a `project.migrated` event
  (`POST /v1/migrations`; data `{id, files, summary}`), so agents can read it with
  `bridle events --kind project.migrated`. A daemon being down never fails a migration.
- A failing migration records nothing for itself and stops the run; those before it stay recorded.

## Running it

`bridle migrate` (the current repository), `--project NAME` (a registry project), or `--all`
(every project in the registry, one at a time, stopping at the first failure and naming the ones
already done). A project whose daemon isn't running is not in the registry, so `--all` skips it;
run `bridle migrate` in its repo. Commit the changed files and `.bridle/migrations.*` afterward.
