---
id: k562
title: "parses_fast fails under load: a wall-clock bound in a unit test"
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [5heh, b7cz]
---

## The ask


`crates/bridle-spec/tests/spec.rs` `parses_fast` asserts that parsing every fixture 20 times
takes under 1 s of wall-clock time. On a loaded machine it fails (b7cz saw it at load 118; the
restart-port worker's `just check` failed on it on 2026-09-30, and managers now note it as a
known flake before landing). A flake in `just check` costs a worker a re-run, and a manager a
judgement call about whether a red check is really red.

Fix without losing the guard's point (catching a pathological slowdown, e.g. a quadratic
parser): drop the wall-clock bound from `just check`. Either move it behind `#[ignore]` as a
benchmark run on demand, or bound something load-independent (a much larger fixture that must
parse in linear-ish time relative to a small one). Same treatment for any other wall-clock
bound in unit tests found on the way (n6gy lists earlier ones).
