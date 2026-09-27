# Specs to tests: making Gherkin disappear

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
- **Adapters live in stack packs.** `python` ships a small pytest plugin that
  gets scenarios from bridle and binds them to pytest-bdd steps. `typescript`
  ships a vitest equivalent, needed by track-web, harness, otters and file-db's
  TS side.
- **Scenario ids in test results.** Results report by `s-b310`, which lets
  `bridle test --task tw-7fa2` run only the scenarios in that task's impact,
  and `bridle spec coverage` list executable scenarios with no bound test
  (research 11's "147 scenarios with zero links to code").
- **Validation stays strict but gets cheaper.** Rejecting near-misses is the
  valuable part of today's tool, and that moves into the Rust parser. Re-parsing
  the output is no longer needed because the emitter and the validator share
  one AST.
