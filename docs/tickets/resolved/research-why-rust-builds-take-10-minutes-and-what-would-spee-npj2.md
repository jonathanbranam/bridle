---
id: npj2
title: "Research: why Rust builds take ~10 minutes and what would speed them up"
kind: research
opened: 2026-10-07
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [b7cz, z7y5]
tasks: [br-npj2]
closed: 2026-10-09T23:11:02Z
---

## The ask

The human, 2026-10-06 ~10:40 PM ET, verbatim:

> can we do some research on any way to speed up Rust builds, or is this just actually how long they take? ... 10 minutes feels like a long time.

They say they've asked before (b7cz covers the cold-worktree part: warm `target/`, `check-affected`, measured 2026-09-29).

## Ask

Research, no build: where the ~10 minutes goes today and what would cut it. Measure first (the human, 2026-09-29: "measure it when possible before committing to more work").

- Which builds take ~10 min: the self-upgrade's release build (`cargo install`, NUC measured 10m53s cold), a worker's first build, `just check`, CI?
- `cargo build --timings` on the release and debug builds: the slowest crates, the critical path, link time.
- Candidates to weigh with numbers: release profile settings (codegen-units, lto, incremental for the self-upgrade build, `debug = 0` / split-debuginfo for dev), a faster linker (lld / ld-prime on macOS), trimming heavy deps or features, splitting the `bridle` / `bridle-daemon` crates for parallelism, sccache, cranelift for dev builds, a persistent target dir for self-upgrade.
- macOS specifics: syspolicyd / Gatekeeper scanning new binaries (z7y5), Spotlight indexing `target/`.

Output: findings on this ticket, a ranked list of changes with the measured or expected saving, and a recommendation. Follow-up builds get their own tickets.

## Findings, part 1 (before the reboot; machine heavily loaded; part 2 below supersedes where it differs)

Stopped on the human's hold (via manager-2). Not done: cargo-bloat, sccache, lld/mold and cranelift
(none installed; only the stable toolchain exists, cranelift needs nightly; the install question is
with the human, rule missing-tools), LTO, crate splitting, and the Spotlight experiment. Numbers
below are from this Mac (i9-9980HK, 8 cores / 16 threads, 32 GB, Intel), not the NUC. It was loaded
throughout by other agents: the load average (1 min, 16 threads) is written next to each number.
Treat any difference under ~30% as noise. A cold run is a fresh, empty target dir.

### Baseline (cold)

| build | wall | CPU (user) | load | note |
|---|---|---|---|---|
| `cargo build --release` (= self-upgrade's `cargo install` profile) | 5m34s | 59 min | 6.6 to 8.4 | 373 units |
| `cargo build` (dev) | 4m57s | 51 min | 9 to 12 | |
| `cargo build` (dev), redo | 8m14s | 50 min | 6, rising to 52 | same CPU, so the extra is contention |
| `just check` | 10m29s | 36 min user + 10 min sys | ~10, 75 at the end | nextest itself 87.6s of it (1301 tests) |
| `cargo clippy --workspace --all-targets` | 2m53s | 22 min | 4.5 to 7 | |
| `cargo nextest run --no-run`, deps already built | 1m14s | 8 min | 5.9 | the test-profile compile of the workspace |

So "about 10 minutes" is `just check` cold (clippy 3m + test build + the `cargo run` of the spec
check + 1.5m of tests), or a loaded cold build. A quiet cold release is about 5.5 minutes here. The
NUC's 10m53s for the release build is about 2x this Mac, consistent with its CPU.

Steady state is much better. The self-upgrade already uses a persistent target dir
(`upgrade-target`, `crates/bridle-daemon/src/upgrade.rs:250`), so only its first build is cold.
After touching `bridle-daemon`: release rebuild 2m20s and 2m18s (load ~10); dev rebuild 12s.

### Where the time goes (`--timings`)

- **Third-party dependencies are most of it.** The release CPU sum is 31 min of unit time; the AWS
  stack used only by `bridle-mail` (2,600 lines of ours) is 43% of it (release) and 37% (dev): 76
  crates exclusive to it, incl. `aws-sdk-s3` 113s, `aws-sdk-sesv2` 86s, `aws-sdk-sts` 26s,
  `aws-config` 21s, and `aws-lc-sys` build script 108s (C). It is also on the critical path: in the
  release build `aws-sdk-s3` ends at 266s and the final `bridle` crate then takes 68s.
- That stack drags a second HTTP stack: `hyper` 0.14 + `h2` 0.3 + `rustls` 0.21 + `http` 0.2
  alongside reqwest's `hyper` 1 / `h2` 0.4 / `rustls` 0.23 (`cargo tree -d`: 20 duplicated crates,
  most of them this pair of stacks). `rustls` 0.21 36s, `h2` x2 57s, `hyper` 0.14 26s.
- `libsqlite3-sys` (bundled) build script: 83s, runs early so mostly off the critical path.
- Our own crates, serial chain: `bridle-daemon` (50k lines) 72 to 93s, then `bridle` (23k lines)
  68s, because the bin depends on the whole daemon. Incremental release is exactly this chain
  (72s + 68s = 140s).
- **Link time is not a problem**: replaying the release link command alone takes 0.5s (ld-prime,
  `ld-1115`, is already the macOS default). A faster linker (lld/mold) saves nothing here.
- Dev profile: `[profile.dev.package."*"] opt-level = 1` makes the cold dev build cost about as much as
  the release build (4m57s vs 5m34s).

### Candidates measured

| change | result | verdict |
|---|---|---|
| dev: deps `opt-level=0`, `debug=0` | cold dev 2m12s (load 7.5) vs 4m57s to 8m14s baseline (load 6 to 52); user CPU 13.6 min vs 50 | big win cold; runtime of tests not measured |
| release: `opt-level=1` for `bridle` + `bridle-daemon` | 2m35s vs 2m18 to 2m20s baseline (load 13 vs 10) | no gain: the crates are frontend-bound, not LLVM-bound |
| release: `codegen-units=256` for the same two | 3m50s vs 2m20s (load 8.6) | worse (CPU 1353s vs 830s) |
| faster linker | link is 0.5s | skip |
| persistent target for self-upgrade | already done | nothing to do |

### Ranked recommendation

1. **Cut or shrink the AWS dependency of `bridle-mail`** (expected: the largest single saving,
   up to ~40% of cold CPU and ~2 min off the cold release critical path; derived from the timings,
   not yet built). Options: a cargo feature so a default build can skip `bridle-mail`
   (only people running the email bridge pay), or replace the SDKs with a small SES/S3 client over
   the existing reqwest (drops the duplicate hyper/rustls stack too). Needs a design look first:
   `bridle mail` is shipped, so a feature gate changes what `cargo install` gives you.
2. **Dev profile: stop optimising dependencies** (`opt-level = 0` for `"*"`, `debug = 0` or keep
   `line-tables-only`): measured about -50% cold dev/test build. Check the test-suite runtime
   before adopting (unoptimised sqlite/tokio could slow the 88s of tests; unmeasured). Incremental
   is already 12s so this only helps cold worktrees and `just check`.
3. **Keep using warm target dirs** (b7cz) and `just check-affected`: the cold cost is paid once per
   worktree; nothing here changes that advice.
4. **Release serial chain** (daemon then bin = 140s of a 2m20s incremental): the only lever left is
   splitting `bridle-daemon` into crates that rustc can build in parallel, or moving CLI code out of
   the final `bridle` crate. Unmeasured; a refactor, so probably not worth it until 1 and 2 are in.
5. **sccache** would share third-party compiles across the ~15 worktrees, which is where the cold
   CPU is, but it cannot cache build-script C builds (`aws-lc-sys`, `libsqlite3-sys`), and it is not
   installed. Unmeasured; revisit after 1 and 2 shrink the dependency set.
6. Not recommended: faster linker, release `codegen-units`/`opt-level` tweaks, cranelift (needs
   nightly, and dev incremental is already 12s).

### macOS background (observed, not proven)

During a 6-minute cold dev build the system daemons took: `syspolicyd` +1m45s CPU, `XprotectService`
+1m, `mds` +10s, `mds_stores` +6s. Earlier, during `just check`, `mds` hit ~50% and `syspolicyd`
~42% CPU, with `sys` time of 10 min against 36 min user. Spotlight indexing is on for the volume and
`target/` has no `.metadata_never_index`. The experiment to compare a build with that marker file
(and its effect on `sys` time) was interrupted by the hold; it is the cheapest thing to finish.
z7y5 (syspolicyd) is the same suspect.

## Findings, part 2 (after dalek's reboot; tools installed)

Re-measured on the rebooted machine, which has the devtools scanning exemption. Load is the 1-min
average at the start of the run (8 cores, 16 threads); other agents were mostly idle until the
sccache runs, so these are much cleaner than part 1, though a build raises the load itself.
Every run is cold (fresh target dir) unless it says incremental.

| run | wall | CPU (user) | load at start |
|---|---|---|---|
| release, cold | 6m12s | 58 min | 3.5 |
| release, incremental (touch `bridle-daemon`) | 2m19s | 14 min | 10 |
| release, relink of the bin crate only | 1m10s | 7.8 min | 8.3 |
| same with `ld64.lld` as linker | 1m19s | 7.9 min | 9.5 |
| dev, cold | 5m58s | 53 min | 7.6 |
| dev, incremental (touch `bridle-daemon`) | 11s | 12s | 10 |
| dev, cold, with `.metadata_never_index` in the target dir | 5m56s | 52 min | 8.4 |
| dev, cold, deps `opt-level=0`, `debug=0` | 2m28s | 15 min | 9.6 |
| test build only, that lean profile (deps built) | 1m29s | 10.7 min | 8.6 |
| nextest run, lean profile | 88s (1301 passed) | | |
| nextest run, current profile | 81s (1301 passed) | | |

What the new runs settle:

- **The reboot did not speed up cold builds** (release 6m12s vs 5m34s before; dev 5m58s vs 4m57s).
  Part 1's loaded numbers were not inflated much by contention; a quiet cold release is 5.5 to 6
  minutes here, dev 5 to 6.
- **Lean dev profile is a clear win and costs nothing at test time**: cold dev 2m28s vs 5m58s
  (-60%), and the 1301 tests run in 88s vs 81s (noise-level). The test build adds 1m29s on top.
  This answers part 1's open question.
- **Linker: no gain.** `ld64.lld` was 9s slower than the default ld-prime on the bin crate (the time
  is rustc codegen; link was 0.5s). Dropping it.
- **Spotlight: no measurable effect.** The `.metadata_never_index` marker changed nothing (5m56s vs
  5m58s). Not worth adding.
- **syspolicyd still shows up, but small**: during the 6-minute cold release `syspolicyd` used +34s
  CPU and `XprotectService` +35s (before the reboot, a 6-minute dev build cost +1m45s and +1m);
  `mds_stores` +2s. Kernel `sys` time was 150s of 3,640s total CPU (4%). It is not what makes the
  build slow.
- **cargo-bloat (release binary, 29.8 MiB of .text)**: `bridle_daemon` 21.7%, `std` 10%, `bridle`
  8.1%, `bridle_gateway` 4.8%, then the AWS stack together about 20% (`aws_lc_sys` 5.8%,
  `aws_smithy_http_client` 3.8%, `aws_sdk_sts` 3.5%, `aws_sdk_s3` 2.7%, `aws_config` 1.5%, ...)
  plus `rustls`/`h2`/`hyper`, against `bridle_mail` itself at 0.9%. So the AWS SDKs weigh about
  20% of the binary for a 2,600-line crate, which backs up the build-time finding.
- **sccache: no measured benefit, and a trap.** Dev cold build into an empty cache: 7m14s; into a
  second fresh target dir with the cache now warm: 6m19s, against 5m58s with no wrapper (both
  sccache runs started at load 56 and 9, so the comparison is rough). The cache had 410 hits, all
  of them C/C++ and assembler (`aws-lc-sys`, `libsqlite3-sys`); Rust units got 0 hits out of 570
  requests, because each worktree has a different absolute path to its target dir and
  those paths are in sccache's cache key. It would need a shared `CARGO_TARGET_DIR` path or path
  remapping across worktrees to help, and workspace crates are never cached while incremental is on.

### Final ranking

1. **AWS SDKs in `bridle-mail`** (about 40% of cold CPU, on the release critical path, 20% of the
   binary, plus a duplicate hyper/rustls stack): a feature gate or a small reqwest-based SES/S3
   client. Expected: cold release 6m -> about 3.5 to 4m. Not built; the saving is derived from
   `--timings` unit durations.
2. **Lean dev profile** (`[profile.dev.package."*"] opt-level = 0`, `debug = 0`): measured
   cold dev -60%, `just check` cold should drop by roughly 3 to 4 of its 10 minutes (its two dev-profile
   builds), tests unchanged. A two-line change in the root `Cargo.toml`. Do this first: it is
   measured, free and independent of 1.
3. **Warm target dirs and `just check-affected`** (b7cz): unchanged advice; cold cost is per worktree.
4. Splitting `bridle-daemon` for parallelism (release incremental is a serial 72s + 68s chain):
   unmeasured, a refactor; defer until 1 and 2 are in.
5. Not worth doing, measured: a faster linker (lld), Spotlight exclusion, release `opt-level` /
   `codegen-units` changes, sccache as set up per worktree. Cranelift was not tried (needs a nightly
   toolchain, and dev incremental is already 11s).

## Part 3: answers to the human's scope change (2026-10-07 ~7:30 AM)

Scope as the human restated it: "ignoring agent time, what are the mechanical reasons that
builds/tests/everything takes so long and uses so much CPU". The reader is new to Rust.

### The human's analysis (pasted from another session, kept verbatim)

> In Rust, each test target is compiled into its own executable, and this workspace has a lot of test targets. In the benchmark's build directory (wt/buildtime/target-dev) I counted 73 test binaries:
> - One per crate for unit tests. The #[cfg(test)] modules inside each crate's src/ are compiled into one test binary per lib or bin target. With 8 crates, that's about 9 of the 73.
> - One per file in tests/. Cargo treats each tests/*.rs file as a separate crate, so each becomes its own executable. There are 64 of them: bridle-daemon has 43, bridle 14, bridle-claude 3, bridle-mail 3 and bridle-spec 1.
>
> The build also creates other new files that get scanned before nextest starts:
> - 28 build-script binaries (build.rs), each compiled and run once during the build.
> - 21 proc-macro dylibs (serde_derive, clap_derive and so on), loaded into rustc as it compiles.
>
> Why they count as new so often:
> - Changing one crate relinks everything that depends on it. Touching bridle-daemon relinks its 43 test binaries plus everything in bridle that uses it. Each relinked binary has a new code hash, so macOS scans it again.
> - A new worktree or target dir makes everything new, including the build scripts and proc-macro dylibs.
> - Debug test binaries are large (tens of MB with debug info), and the scan takes longer on bigger files.
>
> How this adds up to the 84 seconds: before running anything, nextest launches every test binary once with --list to find the tests in it. That's the first launch, so each binary waits for its scan there. Those were the processes we saw sitting at 0% CPU. 73 binaries at roughly 1 s each matches the 84 s after compiling finished. Once listed, nextest starts a new process for every test, but those reuse already-scanned binaries, so they're fast.
>
> Two ways to reduce it:
> - Exempt the build environment, as we've been doing. That removes the per-binary scan cost entirely.
> - Combine the tests/ files into one binary per crate. This is a common Rust layout: a single tests/it/main.rs that pulls the other files in with mod. bridle-daemon would go from 43 test binaries to 1. That also cuts link time, which is a large share of the build on its own, and it would help on any machine, exempt or not. It's a real change to the repo layout, so I can file it as a ticket if you want it considered.

My count agrees: 73 test executables in a workspace test build (checked on the rebooted machine).

### Is the developer-tools scanning exemption a critical perf fix for us?

**No, not for build time; yes, it removed the one place it showed.** The numbers:

- **Listing 73 test binaries now takes 4.5s** (`cargo nextest list`, first run on freshly built
  binaries, load 13) and 3.0s on a second run. That is about 0.06 s per binary, against the human's
  ~1 s per binary (84s) before the exemption. I did not time the list phase before the reboot, so
  the "before" is the human's figure, not mine. The exemption is clearly working for that phase.
- **Nothing else I measured moved with the reboot.** Cold release 5m34s before vs 6m12s after; cold
  dev 4m57s vs 5m58s; the 1301 tests took 87.6s before and 81 to 88s after (that is test execution,
  after listing). Both cold builds were within noise or slower, because the load differed.
- **CPU time spent in the scanners was small even before.** Release build before the reboot: kernel
  `sys` time 164s of 3,700s (4.4%); after: 150s of 3,640s (4.1%). `syspolicyd` use during a 6-minute
  cold build: +1m45s before the reboot (dev build), +34s after (release build). One earlier run, the
  full `just check`, had much more `sys` time (598s against 2,184s user, 21%), which fits many short-lived
  test processes; I did not re-run `just check` after the reboot.
- **Does XProtect persist with the exemption?** Yes, it does not go away, but it is small:
  `XprotectService` +35s CPU over the 6-minute cold release build after the reboot (+1m over the
  6-minute dev build before). In a later, contended run (other agents building at load 14 to 21) the
  system-wide totals were `syspolicyd` +2.5 min and `XprotectService` +4.4 min over 5m26s, but that is
  the whole machine, so I do not attribute it to this build.

So: the exemption is worth keeping, and it fixes the test listing phase (about 80s to about 4s, using
the human's earlier figure), but it does not explain the 10 minutes. Compilation CPU does.

### Why Rust builds are slow: a plain-language account

A Rust program is split into **crates**: a crate is the unit the compiler (`rustc`) works on. This
workspace has 8 of ours (the daemon, the CLI, the gateway, ...), but the build also compiles about
370 crates in total, because every library we depend on (tokio, axum, the AWS SDKs, SQLite's C
code, ...) is downloaded as **source and compiled from scratch** into every fresh target directory.
There is no shared binary cache like npm or pip give you. A new git worktree is a new `target/`, so
a new worktree pays for all of it again: that is the cold build (about 55 CPU-minutes, 5 to 6
minutes of wall time on 8 cores). A **warm** build only recompiles what you touched, which is why the
same project rebuilds in 11 seconds in dev.

Three things make each crate slow to compile:

1. **Generics are copied per use ("monomorphization").** Rust code that is generic (async runtimes,
   serde, HTTP stacks, clap) is compiled once for every concrete type it is used with, so a crate
   that looks small can produce a lot of machine code. The AWS SDKs are generated code and the
   largest case here: 76 crates that only `bridle-mail` needs cost about 40% of the CPU.
2. **The optimiser (LLVM) is slow and the front end is serial.** Release mode asks for fast code, and
   for our own two big crates (`bridle-daemon`, `bridle`) the time is mostly the compiler front end
   and cannot be spread over cores: a crate is compiled as a unit, and `bridle` has to wait for
   `bridle-daemon` to finish. That is the serial 72s + 68s chain in every release rebuild.
3. **Some dependencies are not Rust.** `aws-lc-sys` (TLS crypto) and `libsqlite3-sys` (SQLite) run
   a C compiler at build time (about 108s and 83s).

**Linking** is the last step: the compiler's output objects are joined into an executable. For our
`bridle` binary this is 0.5s, so it is not a problem there. It is a problem for tests: **every file
in a crate's `tests/` folder is its own crate and its own executable**, so this workspace builds 73
test executables (64 from `tests/*.rs`), and each one links against the whole library it tests
(`bridle-daemon` is 50k lines). The debug builds of all of them take 7.6 GB of disk. Touching the
daemon library relinks 43 of them.

Where the CPU goes, then: (a) compiling third-party code in every new target dir, (b) AWS SDK
generics, (c) the serial two-crate chain in release, (d) per-test-binary compile and link, (e) the
dev profile optimising dependencies, which nobody needs. Linking and OS scanning are small.

### The `tests/` consolidation (one binary per crate)

What it is: replace `crates/<crate>/tests/*.rs` with `crates/<crate>/tests/it/main.rs`, which pulls
the old files in as `mod` lines, so a crate has one test executable, not one per file. Cargo and
nextest both support it; it is a common layout.

Measured on a lean-profile warm target (load 18, so rough):

- touching **one** `tests/` file in `bridle-daemon` rebuilds in **5.3s** (compile and link of that
  one test crate);
- touching `bridle-daemon/src/lib.rs` rebuilds the library, the library's own unit-test binary and
  all 43 `tests/` executables in **37 to 45s** (user CPU 235s, `sys` 66s).

I did not do the consolidation, so the saving is an **estimate**: cold test build over the existing
dev build is 1m14s to 1m29s (all crates), of which `bridle-daemon`'s 43 files are the bulk. One
test crate compiles its files once and links once, instead of 43 times, so expect roughly one third to one half off the
test-build step (about 30 to 40s cold) and about 20 to 30s off each rebuild after a daemon-library
change, plus the 7.6 GB target shrinking considerably. `nextest list` also drops from 73 to about
14 executables (about 3s saved, less if the exemption is active). Costs: every test file shares
one crate, so a compile error anywhere fails all of that crate's tests, test names gain a `module::`
prefix (specs-check scans test files by scenario id, `just specs-check`, so check it still finds them), and files that use
`mod common;` or `include_str!` relative paths need adjusting. It would help on any machine and
whether or not the exemption is on.

### Conclusion and suggestions (in order)

1. Lean dev profile (measured: cold dev -60%, `just check` cold roughly -3 to -4 min): do first.
2. AWS SDKs out of the default build or replaced by a small client (expected: cold release ~6m to
   ~4m; the biggest lever on both builds and the binary).
3. `tests/` consolidation, one binary per crate (estimated: -30 to 40s cold test build, -20 to 30s
   per daemon change, much less disk). Worth its own ticket; a mechanical but wide change.
4. Keep the scanning exemption: it fixed the listing phase; it is not a build-time lever.
5. Warm target dirs and `check-affected` stay the advice for day-to-day work (b7cz).
6. Later, if still needed: split `bridle-daemon` for parallel compilation (release incremental).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Findings parts 1-3 and the ranked recommendation are in this ticket (lean dev profile, AWS SDKs out of the default build, tests/ consolidation). No build filed yet; file one when build time matters again.
