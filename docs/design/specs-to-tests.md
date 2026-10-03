# Specs to tests: making Gherkin disappear

> **Status (checked 2026-10-03):** Built, not wired in: the Rust parser, `bridle spec export --format gherkin|json [--task|--scenario]`, `bridle spec coverage`, the pytest adapter (`workflow/packs/python/adapters/bridle_specs.py`) and the vitest one (`workflow/packs/typescript/adapters/vitest-bridle/`), both vendored by hand; no onboarded project uses them yet (track-web, bridle-ui and data-contracts have no `design/specs/`) · Planned: `bridle test --task` as its own command (`spec export --task` covers the selection)

The current pipeline commits a generated `.feature` beside every `spec.md`, so
it needs a staleness check (`check-specs.py`), strict input validation, output
re-parsing through `gherkin-official`, and `uv run`. Most of the brittleness
comes from the committed generated file.

- **Parse in Rust, in the binary.** One parser, used by `bridle spec check`,
  the impact registry, id assignment and export. Target: all six repos' specs
  in well under a second.
- **Nothing generated is committed.** Test runners get scenarios at collection
  time:
  - `bridle spec export --format gherkin --out .bridle/cache/features/`
    (gitignored) for runners that need files;
  - `bridle spec export --format json` for adapters that register scenarios
    directly.
  `export` is built (local, no daemon). It refuses to run, printing the
  diagnostics, when any spec has errors. Defaults: paths and `--root` as for
  `spec check`; gherkin without `--out` writes `.bridle/cache/features/`, which
  this repo's `.gitignore` covers (`.bridle/cache/`), so a project using it
  should ignore that directory too.
  - **gherkin**: one `<capability>.feature` per spec (the file stem, or the
    directory name for `<capability>/spec.md`). A `Rule:` per requirement (kept
    even with no executable scenario, so the gap shows), a `Scenario:` (or
    `Scenario Outline:` with an `Examples:` table) per **executable** scenario,
    tagged with its `@tags` then its id (`@s-b310`). Non-executable scenarios
    are omitted. The shape is data-contracts' `tools/spec-to-feature.py`, minus
    the Purpose paragraph (the AST doesn't carry it).
  - **json**: `{"version": 1, "specs": [...]}`, every scenario, executable or
    not. Per spec: `file`, `capability`, `title`, `requirements`; per
    requirement: `id`, `title`, `protected`, `traces` (`target`, `hash`),
    `text`, `line`, `scenarios`; per scenario: `id`, `title`, `line`,
    `executable`, `tags` (no `@`), `description`, `steps` (`keyword`
    `Given|When|Then|And|But`, `text`, `line`), `examples` (`header`, `rows`, or
    null). `id` is null until `bridle spec id` assigns it. Fields may be added;
    a rename or removal bumps `version`.
- **Adapters live in stack packs.** `python` ships a small pytest plugin that
  gets scenarios from bridle and binds them to pytest-bdd steps. The python one
  is built: `workflow/packs/python/adapters/bridle_specs.py` (vendored by the
  project, not synced; see its README). It registers one scenario per
  executable scenario, named with its id, tags as markers, examples as
  parametrization, selected by `--bridle-spec` / `--bridle-scenario`; a refused
  export fails collection with the diagnostics. `typescript` ships the vitest
  equivalent, built: `workflow/packs/typescript/adapters/vitest-bridle/`
  (`registerBridleSpecs({ steps })`, see its README), for track-web, harness,
  otters and file-db's TS side.
- **Scenario ids in test results.** Results report by `s-b310`, which lets
  `bridle test --task tw-7fa2` run only the scenarios in that task's impact
  (built as `bridle spec export --task ID` / `--scenario ID`, which narrow the
  export to those scenarios; the adapters pass their selection through it),
  and `bridle spec coverage` list executable scenarios with no bound test
  (research 11's "147 scenarios with zero links to code").
- **Validation stays strict but gets cheaper.** Rejecting near-misses is the
  valuable part of today's tool, and that moves into the Rust parser. Re-parsing
  the output is no longer needed because the emitter and the validator share
  one AST.
