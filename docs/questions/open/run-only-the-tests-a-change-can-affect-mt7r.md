---
id: mt7r
title: Run only the tests a change can affect, by strict modularity
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

Every merge is verified with a full `just check`: fmt, clippy and all ~270
tests across the workspace, run twice by the orchestrator. Many of those tests
can't be affected by a given change. Can bridle's design guarantee which tests
a change can affect, so that verification runs only those?

The human's words, 2026-09-28:

> I'm also wondering if we need to always run all of the tests in any
> situation. Some of these tests sound like they would not be impacted by the
> work you're doing. I'm a big proponent of solid CI/CD and thorough testing,
> but if there is a tradeoff that we can make in the design of the system,
> that could help here; E.g. if we add in modularity such that we can
> guarantee that a change in module A is unrelated to module B then we can be
> sure that running only module A's tests is sufficient. That would require
> strict adherence to modularity but would improve performance and likely
> improve coding effort and code design as well.
>
> File that for a follow up

## Why it matters

Full runs take minutes each, compete with workers' builds for the CPU, and
make the load-sensitive flakes in
[[two-flaky-test-failures-under-load-f1ky|f1ky]] more likely. The workspace
is already split into crates (`bridle-claude`, `bridle-api`, `bridle-daemon`,
`bridle`), but most of the tests, and most of the change, sit in
`bridle-daemon`.
