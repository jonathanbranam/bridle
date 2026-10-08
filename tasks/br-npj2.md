+++
id = "br-npj2"
title = "Research: why Rust builds take ~10 minutes and what would speed them up"
kind = "research"
state = "integrated"
created_at = "2026-10-07T10:15:45.617Z"
updated_at = "2026-10-08T01:45:52.124720Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/buildtime"
commit = "7e5e69a5eedbdad48f2bcf7e5a808bb71ebd5fc0"
summary = "Research done; ticket npj2 has parts 1 to 3 (only file changed). Cold on rebooted 8-core Intel Mac: release 6m12s, dev 5m58s, just check ~10m; incremental release 2m19s, dev 11s; link 0.5s. Ranked: lean dev profile (measured -60% cold dev, tests unchanged); AWS SDKs in bridle-mail (~40% cold CPU, est. cold release 6m to 4m); tests/ consolidation (estimate -30-40s cold test build, -20-30s per daemon change; one test file touch = 5s, daemon lib touch = 37-45s); keep scanning exemption (nextest list of 73 binaries now 4.5s; not a build-time lever, sys time ~4%, XProtect +35s small). No gain: lld, Spotlight marker, release opt tweaks, per-worktree sccache. Part 3 adds a plain-language why-Rust-is-slow account."
ticket = "npj2"
+++

Ticket: docs/tickets/open/research-why-rust-builds-take-10-minutes-and-what-would-spee-npj2.md (read it, plus b7cz in docs/tickets/resolved/ and z7y5). Research only: NO code changes, no build.
Goal: find where the ~10 minutes of Rust build time goes and what would cut it. Measure FIRST, then recommend.
Do: time a cold release build (`cargo build --release`, what the self-upgrade runs via `cargo install`), a cold debug build, and `just check`, each in a fresh worktree target dir; run `cargo build --timings` on release and debug and read the HTML for the slowest crates, the critical path and link time. Then measure the candidates one at a time against that baseline: release profile (codegen-units, lto, incremental), a faster linker on macOS (ld-prime / lld), `debug = 0` / split-debuginfo for dev, trimming heavy deps or features (cargo tree -d, cargo bloat if installed), splitting crates for parallelism, sccache, cranelift for dev, a persistent target dir for self-upgrade; macOS specifics: syspolicyd/Gatekeeper scanning new binaries, Spotlight on target/.
If a tool is missing (cargo-bloat, sccache, lld), stop and ask the manager; do not substitute (rule missing-tools).
Output: findings written on the ticket (the baseline numbers, a ranked list of changes with measured or expected saving, a recommendation). Follow-up builds are filed as their own tickets by whoever reads it, not built here.
Acceptance: the ticket has measured numbers for each build type and a ranked list; no source changes in the diff other than the ticket.
Model: Sonnet. Researcher role if the manager has one.

## Thread

### note · agent:manager-2 · 2026-10-07T10:54:14.538Z
br-npj2 (build-time research): cargo-bloat, sccache and lld/mold are not installed on this machine; the brief measures all three. May the worker install them (cargo install cargo-bloat; brew install sccache lld)? Recommend yes: they are standard dev tools and the research is incomplete without them. The worker will do the baseline timings meanwhile.

### note · agent:manager-2 · 2026-10-07T11:16:11.664Z
From orchestrator: machine load is high (5.4/core) so timings are noisy. Record the load (uptime) next to each timing, and redo the key cold and incremental runs when load is below about 1 per core.

### note · external:aide · 2026-10-07T12:13:47.874Z
From the human, via aide (2026-10-07 ~7:00 AM ET), re br-npj2 tool installs (cargo-bloat, sccache, lld): "for npj2 - I approve this installation; can we have the orch do the installs required first and then the worker does not have to?" So: all three approved; the orchestrator installs them, not the worker.

### note · external:aide · 2026-10-07T12:19:25.791Z
From the human, via aide (2026-10-07 ~7:30 AM ET): SCOPE CHANGE and input for br-npj2 / ticket npj2.

The human: "Add this information to research into the rust research ticket as part of the analysis and include any suggestions in the conclusion. Is this dev tools excemption a critical perf. improvement for us? this expands the scope of research beyond just builds. the scope should be "ignoring agent time, what are the mechanical reasons that builds/tests/everything takes so long and uses so much CPU" - I don't know how Rust works and haven't used it so I don't have experience here to draw on."

So: (1) new scope, quoted above; write the findings for a reader new to Rust. (2) Answer explicitly whether the developer-tools (scan) exemption is a critical performance fix for us, with numbers. (3) Put suggestions in the conclusion, including the tests/ consolidation below. (4) Add the following analysis (pasted by the human, from another session) to the ticket:

---
In Rust, each test target is compiled into its own executable, and this workspace has a lot of test targets. In the benchmark's build directory (wt/buildtime/target-dev) I counted 73 test binaries:
  - One per crate for unit tests. The #[cfg(test)] modules inside each crate's src/ are compiled into one test binary per lib or bin target. With 8 crates, that's about 9 of the 73.
  - One per file in tests/. Cargo treats each tests/*.rs file as a separate crate, so each becomes its own executable. There are 64 of them: bridle-daemon has 43, bridle 14, bridle-claude 3, bridle-mail 3 and bridle-spec 1.
The build also creates other new files that get scanned before nextest starts:
  - 28 build-script binaries (build.rs), each compiled and run once during the build.
  - 21 proc-macro dylibs (serde_derive, clap_derive and so on), loaded into rustc as it compiles.
Why they count as new so often:
  - Changing one crate relinks everything that depends on it. Touching bridle-daemon relinks its 43 test binaries plus everything in bridle that uses it. Each relinked binary has a new code hash, so macOS scans it again.
  - A new worktree or target dir makes everything new, including the build scripts and proc-macro dylibs.
  - Debug test binaries are large (tens of MB with debug info), and the scan takes longer on bigger files.
How this adds up to the 84 seconds: before running anything, nextest launches every test binary once with --list to find the tests in it. That's the first launch, so each binary waits for its scan there. Those were the processes we saw sitting at 0% CPU. 73 binaries at roughly 1 s each matches the 84 s after compiling finished. Once listed, nextest starts a new process for every test, but those reuse already-scanned binaries, so they're fast.
Two ways to reduce it:
  - Exempt the build environment, as we've been doing. That removes the per-binary scan cost entirely.
  - Combine the tests/ files into one binary per crate. This is a common Rust layout: a single tests/it/main.rs that pulls the other files in with mod. bridle-daemon would go from 43 test binaries to 1. That also cuts link time, which is a large share of the build on its own, and it would help on any machine, exempt or not. It's a real change to the repo layout, so I can file it as a ticket if you want it considered.
---

### note · external:aide · 2026-10-07T12:19:25.844Z
From the human, via aide (2026-10-07 ~7:30 AM ET): SCOPE CHANGE and input for br-npj2 / ticket npj2.

The human: "Add this information to research into the rust research ticket as part of the analysis and include any suggestions in the conclusion. Is this dev tools excemption a critical perf. improvement for us? this expands the scope of research beyond just builds. the scope should be "ignoring agent time, what are the mechanical reasons that builds/tests/everything takes so long and uses so much CPU" - I don't know how Rust works and haven't used it so I don't have experience here to draw on."

So: (1) new scope, quoted above; write the findings for a reader new to Rust. (2) Answer explicitly whether the developer-tools (scan) exemption is a critical performance fix for us, with numbers. (3) Put suggestions in the conclusion, including the tests/ consolidation below. (4) Add the following analysis (pasted by the human, from another session) to the ticket:

---
In Rust, each test target is compiled into its own executable, and this workspace has a lot of test targets. In the benchmark's build directory (wt/buildtime/target-dev) I counted 73 test binaries:
  - One per crate for unit tests. The #[cfg(test)] modules inside each crate's src/ are compiled into one test binary per lib or bin target. With 8 crates, that's about 9 of the 73.
  - One per file in tests/. Cargo treats each tests/*.rs file as a separate crate, so each becomes its own executable. There are 64 of them: bridle-daemon has 43, bridle 14, bridle-claude 3, bridle-mail 3 and bridle-spec 1.
The build also creates other new files that get scanned before nextest starts:
  - 28 build-script binaries (build.rs), each compiled and run once during the build.
  - 21 proc-macro dylibs (serde_derive, clap_derive and so on), loaded into rustc as it compiles.
Why they count as new so often:
  - Changing one crate relinks everything that depends on it. Touching bridle-daemon relinks its 43 test binaries plus everything in bridle that uses it. Each relinked binary has a new code hash, so macOS scans it again.
  - A new worktree or target dir makes everything new, including the build scripts and proc-macro dylibs.
  - Debug test binaries are large (tens of MB with debug info), and the scan takes longer on bigger files.
How this adds up to the 84 seconds: before running anything, nextest launches every test binary once with --list to find the tests in it. That's the first launch, so each binary waits for its scan there. Those were the processes we saw sitting at 0% CPU. 73 binaries at roughly 1 s each matches the 84 s after compiling finished. Once listed, nextest starts a new process for every test, but those reuse already-scanned binaries, so they're fast.
Two ways to reduce it:
  - Exempt the build environment, as we've been doing. That removes the per-binary scan cost entirely.
  - Combine the tests/ files into one binary per crate. This is a common Rust layout: a single tests/it/main.rs that pulls the other files in with mod. bridle-daemon would go from 43 test binaries to 1. That also cuts link time, which is a large share of the build on its own, and it would help on any machine, exempt or not. It's a real change to the repo layout, so I can file it as a ticket if you want it considered.
---

### note · external:aide · 2026-10-07T12:23:04.984Z
HOLD from the human, via aide (2026-10-07 ~7:40 AM ET): "let's put a hold on the rust research until AFTER I've done a full reboot/restart of bridle and picked up the devtools scanning excemption; anything we find here will be moot and research testing would have to be re-done after that." Stop work on br-npj2 now; resume only after the human's evening restart/reboot (br-8b52, after 5 PM ET). The tool installs can wait too.

### note · external:aide · 2026-10-07T12:23:05.060Z
HOLD from the human, via aide (2026-10-07 ~7:40 AM ET): "let's put a hold on the rust research until AFTER I've done a full reboot/restart of bridle and picked up the devtools scanning excemption; anything we find here will be moot and research testing would have to be re-done after that." Stop work on br-npj2 now; resume only after the human's evening restart/reboot (br-8b52, after 5 PM ET). The tool installs can wait too.

### note · external:aide · 2026-10-07T12:23:22.277Z
Correction from the human, via aide (~7:45 AM ET): "tool installs can complete as they are underway". So the orchestrator finishes installing cargo-bloat, sccache and lld; only the br-npj2 research stays held until after the evening restart.

### note · external:aide · 2026-10-07T12:23:22.327Z
Correction from the human, via aide (~7:45 AM ET): "tool installs can complete as they are underway". So the orchestrator finishes installing cargo-bloat, sccache and lld; only the br-npj2 research stays held until after the evening restart.

### note · agent:manager-2 · 2026-10-07T12:23:28.955Z
HOLD from the human (via orchestrator): stop work on br-npj2 now. No further builds or tool installs. Leave your notes so far on the ticket (numbers with load), commit them, write a short task summary of where you stopped, and message me. Resume only after the human's evening restart (after 5 PM ET).

### note · agent:buildtime · 2026-10-07T12:25:41.679Z
held as asked: partial findings (baselines, timings, ranked list) on ticket npj2, summary written; commit cec00625 (ticket only, main already merged, no just check run: docs-only diff)

### note · agent:buildtime · 2026-10-07T12:25:45.776Z
Held per the human. Findings so far are on the ticket (numbers with load). Builds stopped, my target dirs deleted. Resume after the evening restart.

### note · external:orchestrator · 2026-10-07T22:32:47.706Z
Hold lifted: the human's evening restart/reboot of dalek is done (uptime 5 min at 18:32 ET), sccache, ld64.lld and cargo-bloat are installed (/usr/local/bin, ~/.cargo/bin). Resume from the partial findings on the ticket (cec00625 on the buildtime branch). Measure on the rebooted machine so the devtools scanning exemption is part of the baseline; say in the findings whether syspolicyd scanning still shows up.

### note · external:orchestrator · 2026-10-07T22:32:47.725Z
From orchestrator: br-npj2 hold lifted (dalek rebooted, tools installed; see the thread). Resume it now: respawn or resume worker buildtime on its branch. Two workers max.

### note · agent:manager-2 · 2026-10-07T22:32:51.225Z
Hold lifted (dalek rebooted, tools installed; see the thread comment from the orchestrator). Resume br-npj2: continue from your partial findings, install nothing further without asking, record load next to each timing, redo key cold/incremental runs when load is below ~1 per core, measure the tool candidates (cargo-bloat, sccache, lld). Same acceptance: only the ticket in the diff, summary updated, then message me.

### note · external:orchestrator · 2026-10-07T23:14:31.920Z
Orchestrator observation, 19:14 ET during your cold builds (uptime ~50 min after reboot): XprotectService (pid 667) at ~116% CPU, up 46 min, so it is busy for most of the time since boot; iTerm2 also ~113%. Load peaked at 95 (5.9/core). The daemon's load note blamed python/ps, but the rustc fan-out plus XProtect is the real load. Please include in the findings: does XProtect scanning of build outputs persist with the devtools exemption in place (it's separate from syspolicyd's Gatekeeper check), and how much build time it costs (e.g. compare a build with target/ excluded or in a Spotlight/XProtect-ignored location, if that's measurable without changing the system). Don't change system settings; recommend.

### note · external:orchestrator · 2026-10-07T23:27:27.235Z
Heads-up: from ~7:40 PM ET a second worker will be rebasing and running just check on the parked branches (br-8b98, br-2718, br-2672). Note which of your timings overlap it.

### note · agent:buildtime · 2026-10-07T23:29:09.676Z
Done: final findings on the ticket (part 2 supersedes part 1). Branch has main merged; diff is the ticket only; no just check run (docs-only).

### note · agent:buildtime · 2026-10-07T23:29:11.174Z
done: build-time research on ticket npj2 (ranked: 1 AWS SDKs in bridle-mail, 2 lean dev profile -60% cold measured); f12d4423 (ticket only, main merged, no just check: docs-only)

### note · external:orchestrator · 2026-10-07T23:29:39.604Z
From orchestrator: br-npj2 verified (findings read on f12d4423, ticket-only). Merge it to main and push when you're ready; no just check needed for a docs-only diff beyond what the merge rule requires.

### note · agent:manager-2 · 2026-10-07T23:29:43.293Z
Good work; not landed yet, three gaps against the human's 7:30 AM scope change (see the aide note on the task thread). Add a short final section, no new heavy measurement beyond what is cheap: (1) answer explicitly: is the developer-tools scanning exemption a critical perf fix for us? yes/no with your numbers (before vs after reboot, syspolicyd/XProtect CPU, nextest --list phase if you can time it: 73 test binaries). (2) the tests/ consolidation (one test binary per crate via tests/it/main.rs; bridle-daemon 43 test files): put the human's pasted analysis on the ticket and add it to the ranked list with an expected saving (link count/test build time; measure test-build time on a crate or estimate and say which). (3) a plain-language 'how Rust builds work and why it is slow' paragraph or two for a reader new to Rust (crates, per-test-binary linking, generics monomorphization, deps compiled from source), and the XProtect question from the orchestrator (does XProtect persist with the exemption: your +35s says small; state it). Ticket only in the diff. Commit, merge main, message me.

### note · agent:buildtime · 2026-10-07T23:43:04.939Z
Part 3 added to the ticket (exemption verdict with numbers, human's analysis verbatim, tests/ consolidation with estimate, plain-language account, XProtect answer, conclusion). Consolidation saving is an estimate (not built); everything else measured.

### note · agent:buildtime · 2026-10-07T23:43:13.935Z
done: part 3 on ticket npj2 (exemption: no for build time, fixed nextest list to 4.5s; tests/ consolidation est. -30-40s cold test build; why-Rust-is-slow section; XProtect small); c663ae2b (ticket only, main merged, docs-only so no just check)

### note · agent:manager-2 · 2026-10-07T23:43:26.242Z
integrated: 7e5e69a5eedbdad48f2bcf7e5a808bb71ebd5fc0 (branch bridle/buildtime)

### note · agent:manager-2 · 2026-10-07T23:43:30.728Z
cleanup: removed agent buildtime, branch bridle/buildtime

### note · external:orchestrator · 2026-10-08T01:45:52.124Z
orchestrator, 2026-10-08 ~01:45Z: a load finding for build/test speed. During just check the top consumers are (ps), (bash) and (python3.11): every fake-claude.py spawn goes through the pyenv shim (bash pyenv-exec, then bash pyenv-which, then python3), so each fake agent costs two bash processes plus python. Resolving python3 once (an absolute interpreter path for the fake) would cut that. Not measured; a candidate for the build-speed work.
