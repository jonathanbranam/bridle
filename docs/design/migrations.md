# Project migrations

> **Status (checked 2026-10-03):** Built, not wired in: `bridle migrate` with `.bridle/migrations.toml`/`.log` and the `project.migrated` event (`crates/bridle/src/migrate.rs`); never automatic, by design. Migrations: `0000-baseline` and `0001-rename-product-manager`; bridle's own repo has run both

Some bridle changes alter what a project keeps in its own repo for bridle: ticket frontmatter,
`.bridle/config.toml`, the docs layout. `bridle migrate` brings a project up to date with the
running binary (ticket xebc). The daemon's SQLite database is separate: it migrates itself
([[docs/design/storage|storage]]).

## Rules

- **Automatic by default.** `bridle serve` applies the project's pending migrations at start-up
  (see "At start-up"); `bridle migrate` does the same by hand. A project opts out with
  `[migrations] auto = false` in `.bridle/config.toml`.
- **Opt-in migrations.** A migration with `manual_only: true` is skipped at start-up and by a
  plain `bridle migrate` (listed as pending-manual); it runs only by id, `bridle migrate --only ID`.
  None ships today.
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

## At start-up

`bridle serve` runs the pending, non-opt-in migrations before the daemon starts listening, with
the same code and records as `bridle migrate`; a restart catches up whatever was missed while
down, and an upgraded binary migrates each project as its daemon starts. Nothing here may stop
the daemon: the run is wrapped against errors and panics.

- A failure stops the run at that migration (as `bridle migrate` does), is logged, and files an
  incident task once the daemon is up. The daemon serves regardless.
- A migration refusing for uncommitted changes (`migrate::Refused`) is not a failure: logged,
  no incident, retried at the next start.
- Applied migrations are posted as `project.migrated` events once the daemon is up.
- Nothing pending (or `auto = false`) touches no file.

## Shipped migrations

- **`0000-baseline`**: starts tracking; changes nothing.
- **`0001-rename-product-manager`** (ticket 9j2h): the `product-manager` role became
  `project-manager`. In `.bridle/config.toml` it replaces the name everywhere it appears (the
  `[roles.product-manager]` table, a `system_prompt` path, any value), and it moves a
  `.bridle/roles/product-manager.md` override to `project-manager.md`. It refuses, naming the
  files, when git shows uncommitted changes in any file it would edit, and when both role files
  exist. A project without the role is left alone. Rerunning it is a no-op. **Read-time alias:** the
  daemon reads a stored agent role or a configured role table named `product-manager` as
  `project-manager` (`config::canonical_role`), so existing agents (their stored role is not
  rewritten) and unmigrated projects keep PM permissions until they migrate. This reverses the
  first "no alias" decision: without it, the queue gate and the stop check silently dropped a
  running PM's permissions the moment the daemon updated. Example, before and after:

  ```toml
  [roles.product-manager]                          [roles.project-manager]
  system_prompt = "workflow/base/roles/product-manager.md"   system_prompt = "workflow/base/roles/project-manager.md"
  ```

  Rules that vendor a role list (`roles: [..., product-manager]`) come from the workflow layers
  and are replaced by `bridle init`/sync, not by this migration.
  Existing projects other than bridle's own are migrated only after the human reviews this.

`0001` is flagged `manual_only` (it edits a project's files, and the human reviews it before it
runs on a real project): start-up skips it, and it runs by `bridle migrate --only 0001-rename-product-manager`.
