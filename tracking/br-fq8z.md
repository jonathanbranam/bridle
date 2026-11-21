+++
id = "br-fq8z"
title = "Measure build and test times on the laptop: what makes a change-compile-test cycle take ~an hour"
kind = "research"
state = "claimed"
created_at = "2026-11-21T15:08:46.991Z"
updated_at = "2026-11-21T16:35:58.173181Z"
created_by = "external:aide@vesta"
watchers = [
    "external:aide@vesta",
    "agent:bt-times",
]
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

### note · external:aide · 2026-11-21T15:43:53.766Z
aide: scope extended with the human (commit 153d3c86): CPU-cap runs (-j 2/4/8 under nice) and 5 s CPU sampling as the responsiveness proxy (the human is away this morning). Companion run on remus filed as re-93ym on remus-home. Run on a quiet dalek: hold other worker builds during the measurement, or record load per number.

### note · agent:pm-1 · 2026-11-21T15:43:57.941Z
pm-1: scope extension from the human via aide (commit 153d3c86, in the ticket's ask): add -j 2, 4 and 8 under `nice -n 19`, and 5 s CPU sampling (average and peak idle) as the responsiveness proxy. The human is away this morning and asked for the run now. Coordination, for manager-2 and the orchestrator: hold every other build, just check and worker spawn on dalek during the worker's runs; if that is not possible, record the machine load next to each number. Companion run on remus: re-93ym on remus-home (separate ticket there). The brief above otherwise stands (Sonnet, researcher role, findings on this thread, no code change).

### note · agent:manager-2 · 2026-11-21T15:44:03.348Z
Scope extended (see ticket items 4 and 5, commit 153d3c86): after the cold/incremental runs, repeat the build capped with cargo build -j 2, -j 4, -j 8 under nice -n 19; for each run record wall time and CPU-minutes, and sample system CPU every 5 s (idle %, load average) during the run, reporting average and peak idle left for other processes. Aim: a cap keeping a build near 15 min while leaving most CPU free. Dalek is held quiet for you; record load next to each number anyway. Post when the runs are done.

### note · external:aide · 2026-11-21T16:16:25.883Z
aide: remus companion result (re-93ym, agent:buildtime@remus, m-19187), for comparison. remus = i5-13400F 14 threads, 11 GiB, WSL2 Ubuntu. cold release 2:32 / 22.9 CPU-min (avg idle 35%, low 3%); cold dev 1:50 / 17.0; dev 1-line rebuild 0:05; nextest --no-run cold 2:18 / 24.1; nextest run 1:19 (2021 tests, 77.7s) / 1.9 CPU-min, idle 85%; cold release -j2 nice 6:48 / 12.8 (idle 85%, low 79%); -j4 3:53 / 14.2 (73/69); -j8 2:41 / 17.9 (51/40). Peak RSS 2.8 GiB. Slowest tests: outbox_test and governor_test at 10-22 s each (look like fixed ~10 s waits). Full table in m-19187.

### note · agent:bt-times · 2026-11-21T16:35:58.173Z
bt-times findings for br-fq8z. Machine: dalek, i9-9980HK 8 cores/16 threads, 32 GB, Intel Mac, stable Rust, worktree target dir (not the shared one). Only my runs were building (checked with ps: no cargo/rustc before each start); idle daemons only. Load avg (1 min) at start of each run is in the table: it was 4 to 10 because the load average lags the previous run, not because of other builds. Treat <30% differences as noise.

## Table (dev profile, `--workspace`; profile unchanged since br-npj2)

| run | wall | CPU user / sys | load at start | note |
|---|---|---|---|---|
| dev build, cold, all 16 threads | 7m23s | 57.0 min / 3.6 min | ~2 | br-npj2: 5m58s / 53 min |
| dev build, one-line change in bridle-daemon | 14s | 14s / 4.8s | 8.7 | br-npj2: 11s |
| test compile, `nextest run --workspace --no-run` (deps built) | 6m27s | 41.1 min / 3.7 min | 6.5 | cargo itself says 4m13s; the other ~2m is nextest listing 100+ test binaries |
| test run (2022 tests, 6 skipped, test-threads=8 from .config/nextest.toml) | 2m57s (177s) | 7.7 min / 8.3 min | 4.5 | br-npj2: 1301 tests in 81 to 88s |

Capped cold dev builds, `nice -n 19 cargo build --workspace -j N`, sampled every ~5s (top; machine-wide idle %, 16 threads):

| cap | wall | CPU user / sys | load at start | avg idle | min idle (peak use) |
|---|---|---|---|---|---|
| -j 16 (none, not niced, no sampling) | 7m23s | 57 min / 3.6 | ~2 | n/a | n/a |
| -j 8 | 7m12s | 42.5 min / 2.4 | 9.2 | 51% | 21% |
| -j 4 | 13m57s | 38.9 min / 2.0 | 6.0 | 63% | 44% |
| -j 2 | 20m15s | 33.1 min / 1.6 | 9.7 | 73% | 29% |

Caveats: (1) the -j 8, -j 4, -j 2 runs started with a load average of 6 to 10 left over from the previous run, and I have no idle baseline for an empty machine, so idle % is "left over while building", including whatever the daemons were doing. (2) The samples at 4s sleep + 1s top are coarse. (3) -j 4 min idle 44% vs -j 2 min idle 29% is an artifact of a short burst (a C build script or link), not a trend. (4) User CPU falls with fewer jobs (57 to 33 min): less contention/hyperthread sharing, so CPU-minutes are not a fixed cost on this machine. (5) Release builds were not re-measured; br-npj2's release numbers stand (cold 6m12s / 58 min).

Cap that fits the ask (build near 15 min, most CPU free): -j 4 under nice gives 14 min and leaves 63% idle on average (at least 44% at peak). -j 8 under nice costs the same wall time as no cap (7m12s) and still leaves half the CPU, so it is the better default if "7 minutes at half the machine" is acceptable.

## Slowest tests (per-test times from nextest libtest-json-plus; sum of all per-test times is 1397s over 177s wall)

Distribution: 440 tests take >1s, 224 take >2s, 33 take >5s. Median is far below 1s.

| rank | time | test |
|---|---|---|
| 1 | 24.4s | bridle-daemon outbox_test: a_wake_on_a_remote_project_is_forwarded_home_once_as_a_system_message |
| 2 | 17.3s | outbox_test: a_wake_is_not_forwarded_to_a_visiting_orchestrator_when_there_is_a_local_one |
| 3 | 16.7s | bridle-daemon lib: warm_build::tests::prune_after_land_clears_a_bloated_target_without_a_warm_build |
| 4 | 16.7s | bridle project_resolution_test: the_flag_wins_over_the_workspace |
| 5 | 16.4s | project_resolution_test: the_environment_wins_over_the_workspace |
| 6 | 16.0s | project_resolution_test: inside_a_workspace_every_command_acts_on_its_project |
| 7 | 14.0s | project_resolution_test: outside_a_workspace_every_command_refuses |
| 8 | 12.3s | orchestrator_wake_test: agent_wake_serves_the_orchestrators_wakes_with_the_same_payload |
| 9 | 12.1s | outbox_test: a_direct_orchestrator_with_a_home_keeps_its_rights_and_its_wakes_forward |
| 10 | 11.2s | orchestrator_wake_test: only_the_orchestrator_may_wait_and_a_message_wakes_it |
| 11 | 10.9s | dev_site_test: two_sites_log_in_serve_the_fixtures_and_go_down_one_at_a_time |
| 12 | 10.3s | project_resolution_test: the_orchestrator_launches_only_in_the_home |

Slowest binaries by summed test time: bridle-daemon lib unit tests 143s (625 tests), outbox_test 106s (25), bridle cli_e2e 102s (36), project_resolution_test 88s (7), land_test 77s (23), tasks_test 69s (37), upgrade_test 69s (19), session_test 51s (31), bridle bin unit tests 45s (540), principal_wake_test 41s.

Why they are slow: these are not CPU-heavy tests. Most of them start a real bridle daemon (or the bridle binary) per test, which means process spawn, SQLite init, a TCP port and waiting for readiness; several wait on wake/timeouts by design. The run is system-time heavy: sys 8.3 min vs user 7.7 min for the whole run (process spawn and file IO, on macOS where endpoint security scans every new executable). The sampler showed 45% average idle during the test run (min 27%): test-threads=8 on 16 threads, so tests are neither CPU- nor compile-bound. I did not see the machine-wide nextest lock queue (nothing else ran); it would only add wait time when others run.

## What changed since br-npj2 (2026-10-09)

- Still the same: `[profile.dev]` is `line-tables-only` with `[profile.dev.package."*"] opt-level = 1` (Cargo.toml). br-npj2's lean-profile win (cold dev 2m28s vs 5m58s, tests the same speed) was not adopted. The AWS SDK stack of bridle-mail is still there (22 aws-* crates of 430 in Cargo.lock, 2 direct aws-sdk deps), so its ~40% of cold CPU is still there too. I did not re-run `--timings`; I assume the same.
- Grown: cold dev 5m58s to 7m23s (CPU 53 to 57 min), now 118k lines of Rust in crates/*/src; tests 1301 to 2022, test run 81-88s to 177s. Incremental dev stays tiny (11s to 14s). New here: a separate test compile of 6m27s that br-npj2 measured only warm (1m14s to 1m29s with deps already built): in a fresh worktree it is a second large cost, because the `test` profile recompiles dependencies again with the dev-profile settings.

## Answer

Yes, Rust compilation is what eats the CPU and the machine, but only on cold builds. A fresh worktree pays about 7.5 min dev build (57 CPU-min) plus about 6.5 min test compile (44 CPU-min) before the first test runs, on all 16 threads: that is what puts the laptop on its knees. About 100 CPU-minutes of it is third-party code (most of the test compile, per br-npj2, is dependencies and the AWS stack). After that, a one-line change builds in about 14s, so compile is not what makes a warm cycle long. The warm cycle is dominated by the tests: a full run is about 3 min wall with 440 tests over a second each, most of them starting real daemons and processes, and sys time larger than user time. I could not reproduce "an hour" from compile alone: cold build + cold test compile + test run is about 17 min alone on a quiet machine, and 20+ if the machine is shared, so an hour means several of these overlapping (each worker's worktree with its own cold target, plus just check, which does clippy, a test build and the tests again: br-npj2 measured it at 10.5 min cold on a loaded machine) or contention from many agents at once, not one compile.

## Obvious fixes (listed, not built)

1. Lean dev/test profile (deps opt-level 0, debug 0): br-npj2 measured cold dev -60% (2m28s vs 5m58s) and tests the same speed. This also cuts the 6.5 min test compile, which uses the same deps. Biggest single win for cold worktrees. Not re-measured today.
2. Make bridle-mail's AWS stack optional (cargo feature or small reqwest client): ~40% of cold CPU (br-npj2).
3. Share compiled dependencies between worktrees (warm target copy per worker, b7cz) so cold builds are paid once, not per worker; sccache did not help (br-npj2: path in cache key).
4. Cap CPU for human-facing work: run builds under `nice -n 19 -j 8` (7 min, half the machine idle) or `-j 4` (14 min, 63% idle) per the table above; cargo config `build.jobs`. Behavioural change for just/CI scripts, so a ticket.
5. Tests: the top ~35 tests (>5s) and the five binaries above take most of the test time; give the process-spawning tests a shared in-process daemon fixture where possible, and check the fixed sleeps/timeouts in outbox_test, project_resolution_test and warm_build. Raise test-threads only after fixing br-7fr6-type flakiness (the 8 is deliberate).
6. `just check-affected` for the edit loop (already exists); full `just check` only to land.

Not measured: release builds (br-npj2 stands), incremental test compile, effect of the endpoint-security scanner on sys time, anything with other agents running. Raw logs were in /tmp/fq8z on dalek (not kept in the repo). No code or config changes were made; the worktree is clean.
