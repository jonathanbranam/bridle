# bridle_specs: bridle specs as pytest-bdd scenarios

A pytest plugin that replaces data-contracts' `spec-to-feature.py`,
`run-specs.py` and `test_specs_bdd.py`. It runs `bridle spec export --format
json` at collection time and registers one pytest-bdd scenario per
**executable** scenario. No `.feature` file is committed or kept.
Design: `docs/design/specs-to-tests.md` in the bridle repo.

## What you get

- Test names carry the scenario id: `test_s_b310_reach_comes_from_the_engine`.
- A scenario's `@tags` are pytest markers (`-m engine`); the id is a marker too.
- A `*Examples*:` table is parametrization, ids like `[2-3-6]`.
- One `Rule:` per requirement in the generated Gherkin.
- If `bridle spec export` refuses (spec errors), collection fails with its
  diagnostics.

## Install

Adapters are files a project vendors; `bridle sync` doesn't copy them.

1. Copy `bridle_specs.py` to somewhere on `sys.path` for pytest (e.g.
   `tests/`, or set `pythonpath = tests` in `pyproject.toml`).
2. Add `pytest-bdd>=8` to the dev dependencies. The `bridle` binary must be on
   `PATH`, or set `BRIDLE_BIN`.
3. In the `conftest.py` that holds (or sits above) your step definitions:

   ```python
   pytest_plugins = ["bridle_specs"]
   ```

   Under pytest 8+ `pytest_plugins` is only honored in the rootdir conftest.

4. Replace `tests/test_specs_bdd.py` with:

   ```python
   import bridle_specs

   bridle_specs.register(globals())
   ```

   Step definitions are found the usual pytest-bdd way (fixtures in the
   conftest chain, or imported into this module).

## Selection

| | |
|---|---|
| `--bridle-spec-root DIR` | specs directory; env `BRIDLE_SPEC_ROOT`; default `design/specs` |
| `--bridle-spec CAP` | only this capability (repeatable); replaces `run-specs.py --spec` |
| `--bridle-scenario s-xxxx` | only this scenario id (repeatable) |

Both together narrow (a scenario must match both). An unknown capability or
id is an error, not an empty green run. Tags and `-k` still work as usual.
`run-specs.py --change X` has no equivalent: a task edits specs in place on its
branch, so the whole suite is what that branch runs.

## Migrating from spec-to-feature.py

1. `bridle spec import openspec`, then commit the specs under `design/specs`.
2. Delete `tools/spec-to-feature.py`, `tools/run-specs.py`, the `specs` /
   `run-specs` Makefile targets, the staleness check in `check-specs.py` (use
   `bridle spec check`), and every committed `*.feature`. Drop
   `gherkin-official` from the dev dependencies.
3. Install as above. Replace `DATA_CONTRACTS_FEATURES` use with
   `--bridle-spec` / `--bridle-scenario`.
4. Step text matches what the old generator emitted (`\<x\>` escapes in the
   markdown become plain `<x>` placeholders), so existing step definitions
   should bind unchanged.

## Tests

`tests/` runs pytest in a scratch project against a tiny fixture spec. They
skip when pytest-bdd or a `bridle` with `spec export` is unavailable. Run:

```
BRIDLE_BIN=target/debug/bridle python -m pytest workflow/packs/python/adapters/tests
```
