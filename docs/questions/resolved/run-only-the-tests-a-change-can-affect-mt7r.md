---
id: mt7r
title: Run only the tests a change can affect, by strict modularity
opened: 2026-09-28
resolved: 2026-09-29
repos: [bridle]
changes: [523a26e]
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

## What was built (2026-09-28, ticket mt7r)

A practical, bounded fast path for local iteration: `just check-affected
[base]`. It is a crate-level heuristic, not strict modularity — it does not
attempt to guarantee that a change in module A is unrelated to module B
within a crate, only that crate B can't be affected by a change confined to
crate A's directory when B doesn't (transitively) depend on A.

What it does:
- Computes changed files via `git diff --name-only <base> --` (working tree
  vs. `<base>`, default the merge-base with local `main`).
- Maps changed paths to crate directories under `crates/`, resolves the
  crate names and the reverse-dependency closure from `cargo metadata
  --format-version 1 --no-deps` (parsed with `jq`; no new binary or script),
  and runs `cargo nextest run -p <crate> ...` for just that set.
- Falls back to the full `just test` whenever it can't be sure: any changed
  file outside `crates/` (root `Cargo.toml`, `Cargo.lock`, `justfile`, CI
  config, docs, etc.), no merge-base with `main`, a changed file under an
  unrecognized crate directory, or `cargo metadata`/`git diff` itself
  failing. It never silently under-tests.
- Manually verified: a change confined to `crates/bridle-claude/src`
  selects `bridle-claude bridle-daemon bridle` (their actual
  reverse-dependency closure, excluding `bridle-api`/`bridle-tui`); a change
  to `justfile` or `Cargo.lock` triggers the full-suite fallback.

`just check` (the merge gate) is unchanged: it always runs the full suite.
`check-affected` is additive, opt-in, and only for local iteration.

## Still open

The human's actual ask — guaranteed test selection via strict module
boundaries, so that "module A changed" can *prove* "module B is
unaffected" — is broader than this pass and unaddressed. In particular:
- No enforcement that a crate's public API changes are the only way it can
  affect dependents (e.g. behavior changes without a signature change still
  correctly trigger the reverse-dep closure here, but nothing stops a crate
  from being *too* coarse-grained internally — a change to one module inside
  `bridle-daemon` still reruns every `bridle-daemon` test).
- No finer-than-crate granularity (module- or file-level test mapping)
  considered; the crates are uneven in size (`bridle-daemon` holds most of
  the code and tests), so the practical win here is smaller than the ticket
  hopes for on a `bridle-daemon`-only change.
- CI and `just check` intentionally still run everything; whether CI should
  ever adopt an affected-only fast path (with the same fallback discipline)
  is undecided.

Leaving this ticket open for that broader question.

## Resolution

Resolved by 523a26e: `just check-affected`, a crate-level fast path for local iteration.
