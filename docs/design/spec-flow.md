# The spec flow, start to finish

> **Status (checked 2026-10-03):** Built, not wired in: every command on this page (`bridle spec`, `goals`, `arch`, `explore`), `spec coverage`, `spec export --task`, `goals propose` and `arch propose` (these two call the daemon), and the pytest and vitest adapters; no onboarded project uses them yet (track-web, bridle-ui and data-contracts have no `design/specs/`) · Planned: `explore adopt`, converting active OpenSpec changes to tasks

One page for a project adopting bridle's spec tooling. Every command here is
built, local, and makes no daemon call, except `goals propose` and `arch propose`. Details live in
[[docs/design/specs|specs]], [[docs/design/specs-to-tests|specs to tests]],
[[docs/design/goals-tier|goals]], [[docs/design/architecture-tier|architecture]]
and [[docs/design/explorations|explorations]]; `bridle help <command>` is the
reference for flags.

## Adopting, in order

Starting from OpenSpec (`openspec/specs/<capability>/spec.md`):

1. **Check where you are.** `bridle spec check --root openspec/specs`. Fix any
   errors; a requirement without an id is only a warning at this point.
2. **Import.** `bridle spec import openspec --dry-run`, then without
   `--dry-run`. It moves each spec to `design/specs/<capability>.md` and assigns
   ids, with `design/specs/.ids` as the ledger. It changes nothing if any source
   has an error. Commit the moves and the ledger.
3. **Assign ids** to anything added later: `bridle spec id`. Idempotent;
   `--dry-run` shows what would change. Ids are never reused.
4. **Enforce.** `bridle spec check --require-ids` now passes, and stays in your
   check from here on.

Starting from nothing: write specs in `design/specs/`, then steps 3 and 4.

## Wiring `spec check` into the project's checks

`spec check` exits 1 on any error and prints `file:line:col: message`, so it
drops into anything that runs commands. With `just`:

```make
check: specs-check
    # ... the project's other checks

specs-check:
    bridle spec check --require-ids
    bridle explore check
    bridle arch list > /dev/null
    bridle goals list > /dev/null
```

`arch list` and `goals list` exit non-zero on parse errors, so discarding their
output makes them checks. As a pre-commit hook it is the same line:
`bridle spec check --require-ids`. In CI, run the same recipe; nothing needs
the daemon.

## Feeding tests: `spec export`

Nothing generated is committed. Adapters ask bridle for the scenarios at test
collection time:

- `bridle spec export --format json` prints `{"version": 1, "specs": [...]}`
  with every scenario (executable or not), each with its id, tags, steps and
  examples. An adapter (a pytest or vitest plugin) reads it and registers
  scenarios directly, reporting results by scenario id (`s-b310`). The full
  shape is in [[docs/design/specs-to-tests|specs to tests]]; fields may be
  added, and a rename or removal bumps `version`.
- `bridle spec export --format gherkin --out .bridle/cache/features/` writes one
  `.feature` per capability for runners that need files. Gitignore
  `.bridle/cache/`.

`export` refuses to run, printing the diagnostics, if any spec has errors, so
adapters never see half-valid input.

**data-contracts:** this replaces `tools/spec-to-feature.py`. The `.feature`
output has the same shape (a `Rule:` per requirement, executable scenarios
only, tagged by `@tags` then id), minus the Purpose paragraph. The Python
adapter that consumes the JSON is built (`workflow/packs/python/adapters/bridle_specs.py`,
vendored by the project); until data-contracts adopts it,
`spec-to-feature.py`, `check-specs.py` and their tests stay. See
[[docs/context/onboarding-data-contracts]].

## Goals and architecture

- **Goals**: any `*.md` under `design/goals/`; each goal is a `##` heading with
  a `{#g-..}` id and a `firmness · priority · stance` line.
  `bridle goals list [--priority P] [--stance S] [--json]`.
- **Architecture**: any `*.md` under `design/architecture/`; each element is a
  `##` heading with a hand-written `{#a-.. [invariant]}` id.
  `bridle arch list [--invariants] [--json]`. Duplicate or missing ids are errors.

Both take `--root DIR` when the project keeps them elsewhere.

## Explorations

For work that deliberately diverges from the design:

1. `bridle explore new <task-id>` scaffolds `design/explore/<id>/findings.md`
   with `status: open`; fill in `diverges-from` with the `g-`/`a-`/`r-`/`s-` ids.
2. Work on the exploration branch. Only the findings doc merges to main.
3. `bridle explore conclude <id>` or `bridle explore abandon <id>` sets the status.
4. `bridle explore check` validates every findings doc's frontmatter.

## Also built

- `bridle spec coverage [--tests DIR] [--require-all]`: executable scenarios
  whose id appears in no test source.
- `bridle spec export --task ID` / `--scenario ID`: the export narrowed to a
  task's declared impact or one scenario (what `bridle test --task` was to do).
- `bridle goals propose` and `bridle arch propose`: create a task proposing the
  change (daemon call). The human gates they lead to are planned.
- The pytest and vitest adapters, in the `python` and `typescript` packs.

## Not yet (planned)

- Converting active OpenSpec changes to tasks (archived changes stay in git).
- `bridle explore adopt`: needs the arch-revision flow and its human gates.
