# Project migrations

> **Status (checked 2026-10-03):** Built, not wired in: `bridle migrate` with `.bridle/migrations.toml`/`.log` and the `project.migrated` event (`crates/bridle/src/migrate.rs`); never automatic, by design. Migrations: see "Shipped migrations"

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
  A pending one is announced once (br-df8w, "At start-up").
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
- **Opt-in migrations are announced once.** Pending `manual_only` migrations not yet announced
  produce one chore to-do, filed once the daemon is up and messaged to the project's aide
  (`external:aide`; often the only agent on a project; not the human, not the manager), listing
  the ids and how to run them. The ids are recorded as `announced = [...]` in
  `.bridle/migrations.toml` (committed with the rest by the step below), so the next start files
  nothing; a migration shipped later is announced on its own.
- Applied migrations are posted as `project.migrated` events once the daemon is up.
- Nothing pending (or `auto = false`) touches no file.
- **Commit afterward** (br-rdag): after the run, the start-up commits what it changed: the files the
  applied migrations reported plus `.bridle/migrations.toml` and `.bridle/migrations.log`, in one
  commit `bridle migrate: <ids>`. Only those paths are committed (`git commit -- paths`); the
  human's other dirty files are never staged. The same step is the **catch-up**: if the toml or log
  differ from git at start (an earlier run left them), they are committed with the message "commit
  migration records left by an earlier run". This is what fixes the projects that already carry
  uncommitted migration output (track-web, dalek-home, bridle-ui as of 2026-10-18); a file a
  migration edited before this change (say `.gitignore`) is not guessed at and stays for its owner.
- `bridle migrate` (not `--only`, not `--dry-run`) makes the same commit after its run (br-6gdp) and
  prints the to-do text to stderr where it cannot. `--only` runs leave the commit to the owner.
- **Fail safe.** It commits only on a branch the guard below allows, in a repo rooted at the
  project directory. Where it can't (protected branch, git or a hook refuses) it commits nothing and
  files a chore to-do and messages the project's aide (`external:aide`) naming what is left and the `git add`/`git commit`
  line to run. It repeats at each start until the files are committed.

## Shipped migrations

- **`0000-baseline`**: starts tracking; changes nothing.
- **`0001-rename-product-manager`** (ticket 9j2h): **retired** (br-df8w), removed from the list.
  The daemon reads a stored or configured role named `product-manager` as `project-manager`
  (`config::canonical_role`), so an unmigrated project loses nothing; the rename was cosmetic.
  A project that wants the new name in its config edits it by hand. Rules that vendor a role list
  come from the workflow layers and are replaced by `bridle daemon init`/sync.

- **`0002-standard-ticket-fields`** (ticket syqn, part 3): in every `docs/tickets/**.md` it renames
  `needs`, `see`, `opened`, `filed_by`, `closed` to `blocked_by`, `related`, `created`,
  `created_by`, `resolved` (the new name wins when both are there), and qualifies a bare `id:`
  and the bare ids in `blocked_by`, `related` and `parent` with the project's task prefix
  (`[tasks] prefix`, else the default from the project name). A reference is qualified only when
  this project holds that ticket; an id of another project is left for its owner. A ticket already
  in the new form is skipped, so a rerun finds nothing. It refuses, naming the files, on
  uncommitted changes in a ticket it would edit. Readers keep accepting the old names until every
  project has run it. It is `manual_only`; the owner runs it on the project's trial
  branch with `bridle migrate --only 0002-standard-ticket-fields`, reviews the diff and commits it
  (the migration leaves the commit to the owner).

- **`0003-bug-tickets-that-are-fixes`** (ticket bpku): **retired** (br-df8w), removed from the
  list. It only relabelled `kind: bug` tickets that had a task as `fix`; nothing reads the label
  to behave differently, and `bug` remains a valid kind. Left tickets stay as they are.

- **`0004-ticket-links-in-frontmatter`** (ticket 3v75): each `blocks`, `parent` and `related` edge
  between two tickets of the project (read from `edges.toml` and the rows' `ticket` fields on the
  `bridle/state` branch) is written into the owning ticket's `blocked_by`, `parent` or `related`,
  and the `tasks:` line is dropped from every ticket. A parent already set is kept; edges to ids
  with no ticket file, and `duplicates`, `supersedes` and `discovered-from`, stay edges. Opt-in,
  same review and guard as `0002`; a rerun finds nothing.

- **`0005-a-ticket-for-every-active-task`** (ticket 72t9): every active task (not `integrated` or
  `dropped`) whose `ticket` names no ticket file gets a stub ticket from `bridle ticket new
  --from-task` (it needs the project's daemon, which links the row), and every extra active task
  of a ticket that has another gets one the same way plus `ticket link <new> parent <ticket>` (the
  ticket's first task is the row with its id, else the oldest). Finished tasks stay read-only
  history. Reads the rows from `bridle/state` with git. Opt-in, same review and guard as `0002`;
  the new ticket files are left uncommitted for the owner; a rerun finds nothing. It does not
  move threads: a row whose `ticket` is its own id keeps its thread in `tickets/<id>.md` from its
  next write (storage.md), and `ticket show` finds the others by the row's `ticket`.

- **`0006-backfill-ticket-kind`** (ticket v3dk, br-e7e2): every ticket under `docs/tickets/` with
  frontmatter but no (or an empty) `kind:` gets one: the kind of the row with its id, else of a
  row whose `ticket` names it, else `chore` (edit with `ticket set`). Rows are read from
  `bridle/state` with git, so a stopped daemon changes nothing. Runs automatically; the guard and
  the uncommitted-changes refusal apply; a rerun finds nothing. `ticket check` treats a missing
  kind as an error from here on. Spikes are not ticket-checked and are left alone.

- **`0007-gitignore-agent-site`** (ticket tgxy): appends `.agent-site/` to the project's
  `.gitignore` (creating it) unless a line already ignores it, so throwaway dev sites are never
  committed. Runs automatically; the guard and the uncommitted-changes refusal apply; a rerun
  finds nothing.

**Protected-branch guard** (rule `existing-projects`; `migrate::refuse_on_protected_branch`, the
first one): a migration that rewrites a project's files calls it before writing and is refused
(`migrate::Refused`) while `main`, `dev` or the configured `[branches] release` is checked out,
so it only lands on the trial or integration branch. A branch the project declares as its
`[branches] integration` (set in the config, not the `main` default) is allowed unless it is also the
release branch, so track-web on `dev` with `integration = "dev"` migrates. Since br-p4f2 the guard
reads that setting; on dalek-home the 2026-10-17 `0007` run on `main` was allowed because the
guard treats a declared integration of `main` as the project's own branch, and a project with no
`integration` set is refused on `main`. The start-up commit uses the same guard. Bridle's own repo (project `bridle`) is
exempt. `0002`, `0004` to `0007` use it.
