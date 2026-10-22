# Overnight load on dalek, 2026-10-09 (read-only investigation)

Observed 03:00Z-03:48Z (23:00-23:48 ET). Nothing was changed, killed, or messaged. Times are
UTC unless marked ET.

## Short answer

The load is real work, not one runaway process, and three things multiply each other:

1. **Too many cold, uncoordinated Rust builds at once.** Two new workers (schedmsg spawned
   01:35Z, designfix 02:30Z) each started with an empty `target/` because
   `[worktrees] warm_target = false` (turned off in 2f6860c0 for incident z7y5). Each one does a
   full debug build, including the C dependencies, then `just check` (clippy build, then a
   separate nextest build), and then reruns it. On top of that the daemon's **self-upgrade**
   `cargo install` (release, opt-level 3, not niced) runs after each land.
2. **The CPU is thermally throttled to 20 %.** This is a MacBookPro16,1 (i9-9980HK). `pmset -g
   therm` reports `CPU_Speed_Limit = 20` at 23:44 and 23:47 ET. The log also has repeated
   "Dark Wake Thermal Emergency" sleeps (20:57, 22:26, 22:31, 22:32 ET) and 86 sleep entries on
   10-08/09. At a fifth of normal clock speed every job runs about 5x longer, so jobs pile up in
   the run queue and the load average stays high.
3. **Every newly linked binary gets a security scan.** Each linked test binary has a new ad-hoc
   signature (`flags=0x20002(adhoc,linker-signed)`), so syspolicyd, XprotectService and
   socketfilterfw scan it on first exec or listen. One rebuild relinks about 50-75 test
   binaries of about 59 MB each.

The `ps` storm from n4w4 is now **minor**. The schedmsg test daemons include br-9z2n and do not
fork `ps`. Only the long-running daemons still do (see cause 6).

## Evidence

### The load timeline matches the schedmsg cold build

- The first load-hold note was at 01:56:48Z, at load 40.0, with top consumers "ld 487%, rustc 412%, syspolicyd 147%".
  Later notes: 02:07Z (rustc, syspolicyd, governor_test), 02:33Z (rustc 630 %, ld,
  XprotectService 96 %), 02:55Z (rustc 433 %, clang 251 %). These are from the `messages` table,
  read-only.
- schedmsg's `target/` was created at 21:38 ET (01:38Z). Its test binaries were linked in two
  waves: 46 at 21:55-21:57 ET (01:55-01:57Z, the moment load crossed the threshold), then 75 at
  22:05-22:11 ET after an edit relinked everything. Today `wt/schedmsg/target` holds 17 GB and
  123 executables, 7.2 GB of them freshly signed and needing a scan.
- designfix's `target/` was created at 22:30 ET (02:30Z, at its spawn). Its first `just check`
  (pid 41853) ran about 50 min. At 28 min it was still inside the `libsqlite3-sys` and
  `aws-lc-sys` build scripts: one `clang -cc1` on the bundled sqlite3.c had run 28 min, against
  about 1 min on an idle machine. Its `target/` held only 609 MB after about 50 min. The check
  failed or finished, and the agent started a second `just check` at about 03:21Z.
- schedmsg's first `just check` (pid 39330) also took about 50 min. It started a second run at
  03:21Z. designfix's branch (`36e929aa`, merging main into bridle/designer-role) **does not
  contain br-9z2n** (018b9cfd), so its test daemons will fork `ps` at 200 ms when it reaches
  nextest.

### Builds running during the observation

| Who started it (parent chain) | Worktree / target dir | What |
|---|---|---|
| designfix claude (pid 36675) -> zsh -> `just check` | wt/designfix/target (cold) | clippy, then check of the whole workspace, from scratch (aws-lc-sys, libsqlite3-sys, ring) |
| schedmsg claude (pid 95359) -> zsh -> `just check` | wt/schedmsg/target | nextest (8 threads), then a rerun |
| daemon pid 640 -> `cargo install --path crates/bridle` | .bridle/upgrade-src, `CARGO_TARGET_DIR=.bridle/upgrade-target` (6.0 GB, incremental) | release build at opt-level 3. The single `rustc bridle_daemon` was still running after 50 min wall clock, peaking at 900 % CPU. Not niced (NI 0). |

That makes three separate target dirs. Two are cold or nearly cold, and nothing is shared
between them.

### Why clang runs

`rusqlite` uses `features = ["bundled"]` (Cargo.toml:37), which compiles sqlite3.c.
`aws-lc-sys` comes in through rustls's default aws-lc-rs provider, pulled in by the AWS SDK
crates in bridle-mail (aws-config, aws-sdk-s3, aws-sdk-sesv2, feature `rustls`). `ring` is
compiled as well. Every cold `target/` rebuilds all three C libraries. The upgrade target is
warm, so they don't rebuild there.

### macOS security daemons

- Cumulative CPU since boot (28 h): syspolicyd **380 CPU-min**, socketfilterfw **52 CPU-min**,
  mds 26 CPU-min, trustd 20 CPU-min.
- Samples show bursts right after link and test-start waves: XprotectService (pid 667) at 97-135 %
  at 03:23-03:24Z, syspolicyd at 45-79 %, socketfilterfw at 38-50 %.
- Each new test binary is ad-hoc and linker-signed (`.cargo/config.toml` adds
  `-Wl,-adhoc_codesign`). Each one is a new code identity, so it is assessed on first exec.
  Test daemons that bind a socket also trigger socketfilterfw.
- `log show` returned nothing useful for these processes (private or unlogged), so the link
  between scans and individual binaries is a correlation, not proof.
- Spotlight indexing is on for /Volumes/Data, including the `target/` dirs, but mds CPU is
  small.

### Test-suite fork storm (fake-claude via pyenv)

- In 10 s, about 614 new processes appeared. The largest groups: 285 `(bash)` plus 35 `bash`
  (pyenv shims `pyenv-exec` and `pyenv-which`), 77 `git`, 44 `python3`, 31 `env`.
- fake-claude.py starts with `#!/usr/bin/env python3`. That resolves to
  `~/.pyenv/shims/python3`, so every fake agent costs a bash chain before Python even starts.
  Under load the shim alone took more than 4 s per launch.
- `context_governor_test` (`many_concurrent_crossings...`) had up to 28 python3 fake agents at
  once. `governor_test` runs up to 6 copies in parallel (nextest `test-threads = 8`).

### `ps` polling (n4w4 / br-9z2n / br-crht)

- `ps` launches per 10 s: track-web daemon (648) 7, bridle-ui daemon (662) 7, bridle daemon (640)
  3. No test daemon was seen forking `ps`.
- 648 and 662 started 2026-10-07 22:28Z. That is before a1bde105 (3p3h) and 018b9cfd (9z2n), so
  they still run the old fork-every-tick code. Each has one idle manager agent. Self-upgrade
  only restarts the bridle daemon (640), which has re-exec'd several times; the last was
  02:22Z.
- 640 still forks `ps` for `load.rs` top_consumers and some other callers. That is out of
  scope for 9z2n. br-crht (planned) only unifies the native path.

### Orphans and leftovers

- No orphaned test daemons or fake-claude processes (ppid 1) persist. One transient
  `wt/schedmsg/target/debug/bridle` with ppid 1 appeared for about 1 s; it is a CLI e2e test's
  detached daemon.
- Stale worktrees under wt/ (13 of them from Sep 28 to Oct 6) are idle and use no CPU. They take
  only disk space; Spotlight may have indexed them.

### Sleep and wake distort the numbers

- The Mac keeps entering sleep: Idle Sleep, Maintenance Sleep, and Dark Wake Thermal
  Emergency. The orchestrator's `caffeinate -i -t 300` only covers 5-minute windows. My
  1-minute sampler stalled for 18 min (03:24Z to 03:43Z) while the machine slept.
- The daemon logs "the clock jumped: the machine slept" at 02:00, 02:28, 02:51, 03:00 and 03:16Z.
- Wall-clock durations (50-min checks, the 50-min rustc) include sleep. The load spikes right
  after each wake (79-142) are the backlog draining.
- Power was on and off AC in the evening (battery at 18 % at 20:03 ET and 75 % at 21:57 ET). It
  is on AC and at 100 % now.

### Load samples (uptime)

03:00Z 65.0, 03:18Z 100.6, 03:19Z 116.4 (142.8 at about 03:19 in a separate reading), 03:20Z
67.7, 03:21Z 61.6, 03:22Z 40.7, 03:23Z 29.9, 03:24Z 36.2, (asleep), 03:43Z 79.0, 03:44Z 42.4,
03:45Z 105.2, 03:47Z 43.5. The status line said "HOLDING spawns" throughout.

## Recommended fixes, ranked by impact

1. **Share one build cache across worktrees.** Use a shared `CARGO_TARGET_DIR`, or better a
   per-profile shared dir plus `sccache`, so a new worktree doesn't rebuild 400+ crates and three
   C libraries. Alternatively, bring back `warm_target` but clone from a small, pruned seed
   (only `debug/deps` and `debug/build` for dependencies, not the 564K-file integration target
   that caused z7y5). This removes the cold builds that started tonight's load.
2. **Run at most one heavy build at a time across the machine.** Use a lock or semaphore
   around `just check`, nextest and the upgrade build, or `-j` and test-threads scaled to the
   number of active builders. Right now two `just check` runs and a release build each assume
   they own 16 cores.
3. **Make self-upgrade cheap and polite.** Run it niced (`nice -n 19`/`taskpolicy -b`), skip it
   while workers are building or the load hold is on, and coalesce lands so it builds once per
   quiet period. Consider a lighter release profile for the daemon (`opt-level = 2`,
   `codegen-units = 16`, or `incremental = true` for the install profile). The bridle_daemon
   crate alone took 50+ min under contention.
4. **Deal with the thermals.** Speed limit 20 % plus thermal-emergency sleeps cut real capacity
   about 5x. Keep the laptop on a stand or in a cooler spot, on a full-wattage charger. Consider
   lowering cargo `-j` to about 8 on this machine. The load-hold threshold (2.5 per core)
   assumes full clock speed; it could also read `pmset -g therm` and report throttling.
5. **Stop the fake-claude pyenv fork storm.** Have the test harness launch fake-claude with a
   resolved interpreter path (for example, resolve `python3` once with `pyenv which python3` or
   `sys.executable` and exec that), or set `PYENV_VERSION` and put the real `bin` first on
   PATH for tests. That avoids 5-10 bash and coreutils forks per fake agent. Also consider
   capping concurrency in `many_concurrent_crossings`.
6. **Restart the track-web and bridle-ui daemons** (648, 662) so they run the installed
   no-fork binary. They have run 10-07 code since the reboot and still fork `ps` every tick.
   Self-upgrade only restarts the bridle daemon, so other projects' daemons on the same
   machine need either a restart or a fleet-wide upgrade signal.
7. **Merge main into designfix** (or make sure workers branch from a main that has br-9z2n)
   before its nextest phase, so its test daemons don't bring back the 200 ms `ps` polling.
8. **Reduce security scans and indexing.**
   - Add the `wt/*/target` dirs and `.bridle/upgrade-target` to Spotlight privacy (or create
     `.metadata_never_index`).
   - If allowed, register Terminal or the toolchain as a "Developer Tool" in System Settings >
     Privacy & Security, which exempts its child processes from syspolicyd/XProtect first-run
     assessment.
   - A shared target (fix 1) also cuts the number of new binaries.
9. **Prune old worktrees** (13 stale ones in wt/, several merged per `bridle status`). The gain
   is mostly disk space and Spotlight, not CPU.
