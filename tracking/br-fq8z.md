+++
id = "br-fq8z"
title = "Measure build and test times on the laptop: what makes a change-compile-test cycle take ~an hour"
kind = "research"
state = "planned"
created_at = "2026-11-21T15:08:46.991Z"
updated_at = "2026-11-21T15:35:09.873453Z"
created_by = "external:aide@vesta"
watchers = ["external:aide@vesta"]
ticket = "fq8z"
+++

docs/tickets/open/measure-build-and-test-fq8z.md

submitted by external:aide@vesta

**Hold: talk to the human first.** The human, 2026-11-21 (via the notes aide): "file it but tell the aide to talk to me first to clarify what i want". Don't start the research until the human has clarified the ask with bridle's aide.

## The ask (draft, to be clarified with the human)

Measure on the laptop (dalek), and compare with the NUC (vesta):

- incremental compile, full build, and the test suite, split so it's clear which dominates;
- what makes the laptop unusable during it (CPU/RAM, parallel jobs, the link step);
- options to cut it (fewer jobs / codegen units, a faster linker, splitting crates, test selection, building on the NUC).

Research only: no code changes.

## Why (the human's words, 2026-11-20)

"The Rust solution is destroying my laptop, compiling, taking an hour to make a change, compile it, and run all the tests. With Flume, I'm making changes in minutes, and there's no compiling. Obviously, I don't have as many tests, so that's a big part of it, I'm sure, with Rust. I should have the Rust project do some research on how long testing is and how long compiling is, but definitely, compiling that binary just is destroying my laptop. It just puts it on its knees. I can barely use the laptop, and it seems to be compiling. I'm pretty sure not actually testing, but anyway."

Filed by aide (notes) from the human's notes (area/coding/Bridle and Flume.md).

## Thread

### note · external:aide@vesta · 2026-11-21T15:08:46.998Z
submitted by external:aide@vesta

### note · external:aide · 2026-11-21T15:09:09.179Z
aide (bridle): holding for the human. Per the human via aide@vesta: "file it but tell the aide to talk to me first to clarify what i want". Research kind, so nobody readies it; bridle's aide is clarifying the ask with the human and will post the result here.

### note · external:aide · 2026-11-21T15:14:44.231Z
readied by external:aide

### note · external:aide · 2026-11-21T15:14:44.356Z
aide: the human clarified the ask (verbatim in the ticket's ## The ask, commit 3f5729fe): measure one bridle build alone (wall + CPU) and the test run, rank the slow tests; bridle only, no flume comparison yet, no fixes. Prior numbers in br-npj2 to re-use and update. Readied by aide on the human's clarification.

### note · agent:pm-1 · 2026-11-21T15:14:53.660Z
pm-1 brief: model Sonnet; role researcher is fine (no code changes), with shell access to run the builds and nextest on dalek.
- The measurement is only valid on a quiet dalek. manager-2: spawn this worker when no other worker is building or running just check, and hold other spawns while it measures its cold and incremental builds (a few minutes to about 15). Use /usr/bin/time -l (or equivalent) for wall time and CPU; the worker says what else was running.
- The human asked "maybe we have this, please check first": start from br-npj2's findings (resolved ticket, file in docs/tickets/resolved/) and docs/design/dev-speed or build-cost notes if present; report what is still current and what changed (new crates, the two-binary layout, CI settings) rather than re-measuring what is already known.
- Test timings: nextest run with per-test times (for example `cargo nextest run --workspace` with the JUnit or `--message-format libtest-json` output, or `--final-status-level all`), run separately from the compile (build the tests first with `cargo nextest run --no-run`). Ranked list of the slowest tests and test binaries; note which are slow because of waiting (sleeps, timeouts, the machine-wide nextest lock) and not CPU.
- Findings go on this ticket's thread: a short table (build cold, incremental, tests compile, tests run), the top slow tests, and the one-paragraph answer to "is it Rust compilation or something else". Obvious fixes are listed, not built. No flume comparison.
- Acceptance: the findings posted; no code or config change merged.
