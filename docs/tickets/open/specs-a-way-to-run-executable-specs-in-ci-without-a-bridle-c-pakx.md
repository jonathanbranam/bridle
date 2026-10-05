---
id: pakx
title: "Specs: a way to run executable specs in CI without a bridle checkout"
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [75zr]
tasks: [br-pakx]
---

## The ask

From meta-notes-ui's orchestrator (2026-10-05), adopting bridle specs with a vendored vitest-bridle: two frictions, one theme, getting executable specs running in a project's CI without a bridle checkout and without a drifting copy of the test glue. Submitted as br-pakx and br-m4cx (the second is merged here).

1. CI has no bridle binary. vitest-bridle and the pytest plugin read scenarios from bridle spec export --format json at collection time, so the spec tests skip in a project's GitHub CI and a regression can reach main uncaught (workers run them before merge only).
2. The vendored vitest-bridle drifts. The README says copy, symlink or file: dependency; nothing tells the project when bridle's copy or the export format changes.

Options offered: (a) a released bridle binary plus a setup-bridle GitHub Action; (b) a committed export kept in the repo, with bridle spec check failing when it is stale, so CI needs only vitest; (c) publish vitest-bridle (npm or a tagged git dependency); (d) bridle warns when the vendored copy's version differs from its own.

Recommendation (keep it simple; no release infrastructure exists yet, chvf 2 is unbuilt): do (b) and (d). (b) bridle spec export --check <file> (or writes to a path and bridle spec check compares) fails on a stale committed export, and the test runners read the committed file when present, falling back to calling bridle. (d) the export carries a format version, and vitest-bridle and the pytest plugin refuse a newer major with a clear message, which is the drift guard without publishing anything. Defer (a) and (c) until a release channel exists. This is a recommendation for the human to veto, not yet approved.
