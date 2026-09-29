+++
id = "br-b85c"
title = "P3: bridle spec export --task ID selects the scenarios in a task's impact"
kind = "feature"
state = "integrated"
created_at = "2026-09-29T09:12:21.885Z"
updated_at = "2026-09-29T09:34:56.042080Z"
branch = "bridle/export-task"
commit = "37d4ace5fcdab00bc2948965aa30e89ce259569a"
summary = "bridle spec export gains a repeatable --scenario ID (s- keeps that scenario, r- keeps the requirement whole) and --task ID (asks the daemon for the task's declared impact, selects its modify/add-under/remove ids; exits 1 if none declared). Filtering is spec_export::select on the parsed Spec, so json and gherkin share it; requirements and specs with nothing selected are dropped. spec_export is now async (dispatched before the sync spec()). Tests: unit tests for select, e2e for the --scenario filter and --task with a real daemon. Docs: cli.md, specs-to-tests.md, CHANGELOG."
+++

Goal (specs-to-tests.md: 'bridle test --task tw-7fa2 run only the scenarios in that task's impact'): add to `bridle spec export` a repeatable `--scenario s-xxxx` filter (json and gherkin both; requirement kept only if one of its scenarios is selected) and `--task ID`, which asks the daemon for the task's declared impact (bridle impact show, built) and selects the scenarios it modifies/adds-under/removes (a requirement id selects all its scenarios); with --task and no declared impact, exit 1 with a message. The Python and TS adapters (other tasks) can pass their selection through it. Files: crates/bridle spec_export.rs, cli.rs, docs specs-to-tests.md, cli.md.

Acceptance: just check passes; tests: --scenario filter on a fixture; --task path with a stub or e2e daemon if the CLI e2e harness makes that cheap. Model: Sonnet. Out of scope: running tests (`bridle test`).

## Thread

### note · agent:manager-2 · 2026-09-29T09:34:56.042Z
integrated: 37d4ace5fcdab00bc2948965aa30e89ce259569a (branch bridle/export-task)
