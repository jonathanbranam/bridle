+++
id = "br-b85c"
title = "P3: bridle spec export --task ID selects the scenarios in a task's impact"
kind = "feature"
state = "planned"
created_at = "2026-09-29T09:12:21.885Z"
updated_at = "2026-09-29T09:12:24.026005Z"
+++

Goal (specs-to-tests.md: 'bridle test --task tw-7fa2 run only the scenarios in that task's impact'): add to `bridle spec export` a repeatable `--scenario s-xxxx` filter (json and gherkin both; requirement kept only if one of its scenarios is selected) and `--task ID`, which asks the daemon for the task's declared impact (bridle impact show, built) and selects the scenarios it modifies/adds-under/removes (a requirement id selects all its scenarios); with --task and no declared impact, exit 1 with a message. The Python and TS adapters (other tasks) can pass their selection through it. Files: crates/bridle spec_export.rs, cli.rs, docs specs-to-tests.md, cli.md.

Acceptance: just check passes; tests: --scenario filter on a fixture; --task path with a stub or e2e daemon if the CLI e2e harness makes that cheap. Model: Sonnet. Out of scope: running tests (`bridle test`).
