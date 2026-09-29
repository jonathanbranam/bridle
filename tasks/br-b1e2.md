+++
id = "br-b1e2"
title = "P3: bridle spec coverage: executable scenarios with no bound test"
kind = "feature"
state = "planned"
created_at = "2026-09-29T09:12:21.869Z"
updated_at = "2026-09-29T09:25:39.470203Z"
summary = "Implemented `bridle spec coverage` command to list executable scenarios without bound tests. The command scans spec files in design/specs and test directories (default: tests/, test/) for scenario id references, reporting executable, bound, and unbound counts. Supports --root, --tests (repeatable), --require-all, and --json options. Added tests with temp directory fixtures. Commit: eab9c90"
+++

Goal (docs/design/specs-to-tests.md, 'Scenario ids in test results'): `bridle spec coverage [--root DIR] [--tests DIR].. [--json]` (local, no daemon) lists executable scenarios whose id (s-xxxx) appears nowhere in the test sources. Test dirs default to tests/, test/, src/ ... keep the default to `tests` and `test` if present, override with --tests; scan text files (skip binary, node_modules, target, .git) for the token 's-' plus hex ids of scenarios; a scenario is bound if its id appears at least once. Output: counts (executable, bound, unbound) and the unbound list as 'file:line: s-id title'; exit 1 with --require-all if any unbound (default exit 0). Scenarios without ids: reported as 'no id, run bridle spec id'. Files: crates/bridle-spec (small coverage fn using the existing parse), crates/bridle cli.rs/commands.rs, docs specs-to-tests.md, specs.md, cli.md.

Acceptance: just check passes; tests with a temp tree. Model: Haiku. Out of scope: running tests, per-task selection.
