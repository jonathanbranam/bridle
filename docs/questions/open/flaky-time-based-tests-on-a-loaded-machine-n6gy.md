---
id: n6gy
title: Do time-based test assertions need wider margins on a loaded machine?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [zk4p]
---

## The question

Some tests are flaky specifically when the machine is already busy — for
example, timing-sensitive budget-governor staleness tests
(`crates/bridle-daemon/src/governor.rs` and its tests) were observed to pass
or fail depending on how loaded the machine already was at the time.

Do time-based test assertions in this suite need wider margins, a way to run
them serially rather than concurrently with the rest of the suite, fake/
mocked time instead of wall-clock sleeps, or something else?

## Why it matters

Flaky tests that fail only under load erode confidence in `just check`
as a green/red signal (see
[[mergers-must-verify-checks-themselves-zk4p|should merging a worker's
branch require the merger's own passing test run]]), and make it hard to
tell a real regression from machine noise.
