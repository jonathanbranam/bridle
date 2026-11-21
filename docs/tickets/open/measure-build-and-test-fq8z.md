---
id: br-fq8z
title: "Measure build and test times on the laptop: what makes a change-compile-test cycle take ~an hour"
kind: research
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Clarified by the human with bridle's aide, 2026-11-21 (verbatim below). Research only, no code changes. Keep it simple: one build alone on a quiet dalek is enough.

1. **Build**: wall time and CPU for one bridle build (cold and after a one-line change), measured alone, with nothing else building. Re-use the method of br-npj2 (resolved 2026-10-09, `docs/tickets/resolved/research-why-rust-builds-take-10-minutes-and-what-would-spee-npj2.md`) and say what changed since its numbers (cold release 6m12s / 58 min CPU; release incremental 2m19s / 14 min CPU; dev cold ~6m / 53 min CPU).
2. **Tests**: how long the test suite takes to run (separately from compiling the tests), and which tests or test binaries are particularly slow (a ranked list from nextest's per-test times).
3. **The answer the human wants**: what takes the most time and CPU in bridle's change-compile-test cycle, and whether it is Rust compilation itself or something else.

Not in scope now: comparing with flume (bridle first; a direct comparison is not valid yet), runtime performance, and fixes (list any obvious ones, don't build them).

### The human, 2026-11-21, to bridle's aide (verbatim)

"ah, so, the goal is, if we don't know, very simple: measure the timing for bridle builds vs. tests and also break down if there are tests that are particularly slow.

The purpose is this:

I want to determine what is slow and CPU hog for bridle; is it rust itself the problem or something else; I recently started working on flume (you can talk to the aide@flume about it if you want); it doesn't do a fraaction of what bridle does, but it's extremely fast to make changes; I want to determine why. this is about dev cycle time, not runtime performance; I'm sure bridle as a compiled rust binary is faster and more efficient

1. is flume fast because it is small?
2. is flume fast because it doesn't have all the bridle features?
3. is flume fast because it doesn't run all the bridle tests

so, since flume does only a small fraction of this, a direct comparison is not really valid, so let's investigate bridle first and be sure that we understand what takes the most time.

my hypothesis is that the rust build is taking a very long time and we know it spikes CPU and is a drain on dalek resources. i want a clear, simple measurement of this; nothing fancy, just a single build alone is fine; get a measure of CPu as well; maybe we have this, please check first.

then i want to know how long the tests take to run and if there are particular tests that are worse."

## Why (the human's words, 2026-11-20)

"The Rust solution is destroying my laptop, compiling, taking an hour to make a change, compile it, and run all the tests. With Flume, I'm making changes in minutes, and there's no compiling. Obviously, I don't have as many tests, so that's a big part of it, I'm sure, with Rust. I should have the Rust project do some research on how long testing is and how long compiling is, but definitely, compiling that binary just is destroying my laptop. It just puts it on its knees. I can barely use the laptop, and it seems to be compiling. I'm pretty sure not actually testing, but anyway."

Filed by the notes aide (aide@vesta) from the human's notes (area/coding/Bridle and Flume.md).
