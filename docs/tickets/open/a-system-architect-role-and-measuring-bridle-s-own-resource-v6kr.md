---
id: v6kr
title: A system architect role, and measuring bridle's own resource cost against a baseline
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [n4w4, 67qw, 3w9t, ukpm, 6h65, w2hj]
tasks: [br-v6kr, br-re57, br-57ec, br-g9xe]
---

## The ask

The human, verbatim (2026-10-08 ~9:35 PM ET), after the n4w4 postmortem (bridle's own `ps` polling held all spawns for about 26 h, and no agent or role noticed):

> Okay, there are a couple of things here:
>
> * Out of the timed loop. That sounds interesting, but do we have any way of capturing performance characteristics of the program? Do we have to run it? Do we compile with something on to generate that information? Is there something we should schedule once a week, or is there a more direct way to measure things?
> * Forking PS: it's a different process showing up, consuming resources. I don't know if that's traceable. Is it a child process or not? If it is, then we should have caught it. If not, then that's harder to catch. Are we using threads? I don't know how that is going internally in the daemon. Do we have a baseline for how performance is now that we could come back and compare to?
>
> These are the things I would do. I was trying to address this. Again, this is essentially a need for a system architecture role that would consider these types of things, and I don't want to be monitoring everything all the time. That only makes sense while we're building, which is possible, but not as a long-term solution.

## Facts (checked by the aide)
- **`ps` was a direct child of the daemon:** `Command::new("ps")` in `containment.rs:110` (and in `load.rs:44` on each load crossing). So its CPU was the daemon's own children's CPU, and `getrusage(RUSAGE_CHILDREN)` would have counted it. The governor's top-consumer list showed `(ps)` but didn't say whose child it was.
- **The daemon is one process on tokio's multi-threaded runtime** (`#[tokio::main]` in `crates/bridle/src/main.rs`), plus `spawn_blocking` threads. Agents are separate `claude` child processes.
- **There's no performance baseline and no measurement:**
  - no metrics of the daemon's own CPU, forks or tick costs
  - no profiling setup
  - br-9z2n and br-3p3h both skipped the before/after load measurement they were asked for
  - npj2 measured build times only

## The ask
1. **A system architect role:** on-demand, not standing (see w2hj, 6h65). It owns the system's resource behaviour and architecture: periodic loops, child processes, threads, what bridle costs the machine. It reviews designs for those effects, and it runs a periodic check against a baseline, so the human doesn't have to watch everything.
2. **Measure bridle's own cost, and keep a baseline.** The design should cover:
   - the daemon's self and children CPU (`getrusage`), forks per minute and loop tick costs, logged or served as metrics
   - a repeatable idle and one-agent benchmark, run on a schedule (weekly, or on each release), compared with the stored baseline
   - profiling on demand (samply, Instruments, or `tokio-console` behind a build flag), with no always-on cost
3. **Make the first baseline now,** before more fixes land, so they can be compared against it.

## Related
- n4w4: the postmortem. Its recommendations 2 (a resource-budget test) and 3 (an audit of periodic loops) were approved 10-08.
- 67qw: improve the base system's architecture. The human's to-do br-9667 is to expand on it.
- 3w9t: the outside-look role. The human said not to work it yet.
- ukpm: the designer role, approved 10-08.
- 6h65 (product manager) and w2hj (on-demand roles with an automatic stop).

## The human's direction (2026-10-08 ~9:45 PM ET, dictated)

> Okay, that sounds good. We need to just schedule this, obviously, overnight when there's no load. If it takes a day or two to get phase 1 done, that's fine. Hopefully we can do it tonight.
>
> For phase 2, counterstand sounds good. I just log to a file, probably. I don't know why not. That seems easier. I don't know what endpoint it would be calling. It'd be a separate process of the campaign, but I don't want to not use an external hotload solution here.
>
> The repeatable benchmark is good, but that is different from the main run. I'm not sure about the baseline, but it'd be good. It needs to be doing something, right? I'd like to see what that scenario is. It would be something like simulated input on tmux, sending messages between demons or between agents, maybe sending messages between projects and demons. We can skip intermachine stuff, I think, for this.
>
> What else? Starting a background worker, having it do something, shut it down, do that a few times in different parts of the system. I wouldn't run it extensively or repeat it often, but having a baseline and a repeated benchmark once a week would be good. We could add a few different scenarios. It'd only take 10 or 20 minutes each, and then we could add scenarios over time. They'd all be sequential, essentially, so we can't add too many. You can't change them too much over time, or you lose repeatability.
>
> I think that's good. I had another comment, which is how many daemons the daemon starts per minute. That seems like a good metric to always watch, so put that in logs. Memory's good. Machine load is something. What concerns me for some of these is that the process of starting per minute seems like something that could go in a counter. That's what I was trying to say.

The aide's reading. The dictation is garbled in places, so check with the human where it matters.

**Phase 1 (the baseline):**
- Run it overnight when the machine is quiet. A day or two is fine, and tonight if possible.

**Phase 2 (counters):**
- The counters are written to a log file every minute: no endpoint, and no external or hosted metrics service.
- **Processes started per minute** is an always-on counter in that log, with memory and load beside it.
- "counterstand" = counters.
- "I don't want to not use an external hotload solution": the human clarified that it meant writing the counters to a file rather than sending logs to a listener. The "status endpoint" the aide had mentioned meant a route on the daemon's own HTTP API (`bridle status` style), not OpenTelemetry. The decision is file first.

**The benchmark scenarios have to do real work.** Ones the human named, same machine only (inter-machine is out):
- simulated input to an interactive session through tmux
- messages between agents, between daemons, and between projects
- starting a background worker, having it do something, and stopping it, a few times, in different parts of the system

**How the benchmark runs:**
- Each scenario is 10-20 minutes, and they run one after another.
- It runs weekly, against the baseline.
- Scenarios are added over time but changed rarely, so the numbers stay comparable.
- The human wants to see the scenario list before it's built.

## Order, and monitoring as a load source (the human, 2026-10-08 ~9:55 PM ET)

> Okay, the thing to consider here is that, for anything always-on, any continuous monitoring can be a new source of load, right? Our PS problem was part of this. It was monitoring the system, right? All well-intentioned, but too much and incorrectly written monitoring can just cause as much load as anything else, so we need to be careful. What we're doing, I want to establish the baseline, I think, before doing any of these additional enhancements, so we know where we stand right now.

So:
1. **The baseline (phase 1) comes first,** before any counters, benchmark or other enhancement.
2. **Every always-on measure must be shown to be cheap,** checked against the baseline before it lands. For example, the counters read numbers the process already has (`getrusage`, a counter incremented at the existing spawn sites). They never start a process, and never do work proportional to the machine. The containment `ps` poll was well-meant monitoring that became the load (n4w4).

## Memory: which measure (the human, 2026-10-09 ~8:20 AM ET)

The human, verbatim: "in our analysis of memory usage, are we relying on top only or are we using macos specific information? I have this report and am concerned about whether ths impacts the benchmarking and memory monitoring we have planned (or done?)", quoting a report:

> top reports 29 GB used and 3.3 GB free. However, macOS reports 81% free memory and swap use is small (about 960 MB of 2 GB). On macOS, the "used" figure includes file cache that the system frees when something needs it, so memory isn't actually under pressure.

Facts (checked by the aide at dade5b1a):
- Bridle reads no memory figures today. The load governor (`load.rs`, `governor.rs`) uses load average and per-process CPU only. No baseline or counter has been run or built yet.
- So nothing done is affected, but phase 2's "memory beside load" names no measure. top's used/free (and sysinfo's `used_memory`) counts file cache and inactive pages as used on macOS, so it would read near-full on an idle Mac.

For the design: on macOS log the memory pressure level (`kern.memorystatus_vm_pressure_level`, what `memory_pressure` reports), compressed memory and swap used, not used/free. For bridle's own cost, log its processes' footprint or RSS. On Linux, use `MemAvailable` and PSI (`/proc/pressure/memory`).

## The first baseline was lost (2026-10-10)

The 12 h passive sample started at 08:26Z on worker w4vmc, which also held br-4vmc's parked branch.
Landing br-4vmc at ~11:25Z removed w4vmc and its worktree, which held the uncommitted script and
~2 h of CSV. Incident br-vt9k; the landing fix is br-37r9 (landed 5e2944fd, ticket 37r9). The 12 h
duration itself was a misreading: the human's "overnight" (2026-10-08) said when to run, not how
long (the human, 2026-10-10 ~7:20 AM ET: "Why would we sample for more than 30 minutes?").

## The human's plan for the benchmark (2026-10-10 ~9:45 AM ET, via the aide, verbatim)

> wow - we lost everything from the benchmark sampler! OMG that is a huge miss; traack in the
> incident log.
>
> So, sure, let's just run a "live benchmark" today; don't pause any work, don't interrupt
> anything; run a sampler benchmark any time it is ready; the sampler script or whatever MUST be
> committed and merged first; so schedule that separately if necessary or write it first; then
> the sampler/benchmark will run, results are committed! Stored forever (until we purge due to
> irrelevance I guess). If the data sizes are too big for github then suggest an alternative
> solution; they could be stored on a branch for now so we can purge them later without
> re-writing history.
>
> The benchmarking is important but should not interrupt other work during the day; do we have
> the design role? If so, have it do a design pass on the benchmarking to ensure that the design
> is appropriate and considers all critical factors:
>
> 1. benchmark during idle vs. busy
> 2. no fixed scenarios yet
> 3. benchmark should have a log of what happened during the run (event log export?)
> 4. must run the same script every time, so design and write a good script
> 5. commit and merge the script first
> 6. what is the shape and size of the output data and where should it live?
> 7. keep the results permanently somehow; if too big for github, create a folder on dalek inside
>    the bridle workspace parent folder and add benchmarks in dated / timestamped folders; we can
>    send to DropBox or something else later for retention
>
> Since this went poorrly last night, I need to review the updated design and plan. So, do all
> the work to get a plan made; the script can be written that's fine (we can re-write it if there
> are issues). But don't schedule or run the benchmark until I sign off.
>
> This is my plan - send all of this verbatim to the PdM and ask them to put this into an epic in
> the proper theme and to own and balance the schedule for this alongside other work.

## Plan (PdM, 2026-10-10)

Epic "Benchmarking" (theme performance), owned by the PdM. Steps, each its own task:

1. **Design pass** (designer): options and a recommendation written into this ticket under
   `## Design options`, covering the human's seven points. The PdM sends it to the human through
   the aide for review.
2. **The script** (worker, after the design): written to the chosen design, committed and merged
   to main. Allowed before sign-off (the human: "the script can be written").
3. **The first live run**: GATED. It is not scheduled or started until the human signs off on the
   design and plan. Runs alongside normal work, interrupts nothing, and commits its results where
   the design says.

The old phase-1 brief on br-v6kr (12 h passive sample, script uncommitted) is superseded by these.
The architect role and the counters (phase 2) stay as the order above sets them.

## Design options

Focus: internal architecture (where the sampler lives, what it couples, where data lands). There is
no new user-facing CLI in the recommendation; the one "interface" is the script's usage.

**The problem in a sentence:** take a repeatable, cheap, 30-minute reading of what bridle costs the
machine, keep it forever, and never lose it or disturb other work the way the first attempt did.

### What exists today

- No sampler, counters or memory reads in bridle (see Facts and Memory above). Nothing to extend.
- The event log (`bridle events --json`, `--since`, `--kind`) is the daemon's SQLite `events` table;
  it is pruned at 30 days (docs/design/storage.md), so a run's log must be exported to outlive it.
- The state branch `bridle/state` (own worktree, pushed to origin) is the precedent for "data on a
  branch, not on main".
- Two causes of the loss (br-vt9k): the script and CSV lived uncommitted in a worker's worktree, and
  landing removed the worktree. Nothing that matters may live in a worktree or be a child of an agent.

### Decisions common to all options

- **Length and rate:** 30 minutes, one sample every 15 s = 120 rows. Short is enough (the human,
  2026-10-10); 15 s keeps the sampler's own forks to a handful per sample.
- **What a row holds** (all read from outside; the sampler never touches the daemon's write API):
  - time (UTC), load average (1 min);
  - daemon pid; daemon self CPU time and RSS; summed CPU time and RSS of its direct children and of
    the agent `claude` processes, kept as separate columns (so a `ps`-style child shows up as the
    daemon's cost, the n4w4 lesson);
  - count of agents by state (one `bridle status --json`, a read);
  - memory, macOS measures per "Memory: which measure": pressure level
    (`kern.memorystatus_vm_pressure_level`), compressed pages, swap used. Never top's used/free.
    Linux (`MemAvailable`, PSI) is left out until a Linux host needs it (YAGNI).
  - CPU is cumulative CPU seconds, so the analysis takes deltas; the sampler does no maths.
- **Limit, said plainly:** a 15 s sampler cannot see forks per minute (short-lived children vanish
  between samples). Spawns can only be counted by an in-daemon counter (phase 2). The run reports the
  `agent.*` events from the event log as the spawn count, and that is all it claims.
- **Its own cost is measured:** the sampler records its own CPU time in the manifest, so "the monitor
  became the load" (n4w4) is checkable on every run.

### The seven points, and the options

**1. Idle vs. busy.** Two readings are different numbers, and mixing them ruins comparison.
- A. Wait for a quiet window, then run. Cleanest, but "today, interrupt nothing" may never get a
  window on a working day, and it needs a definition of quiet.
- B. Run live whenever ready and record the load context (agents by state per sample, plus mean
  and max at the end in the manifest). Classify afterwards: a run with zero working agents for its
  whole length is "idle"; anything else is "busy". Only like is compared with like.
- Recommend **B** now (it is what the human asked for today), and run an **A-style run later** in a
  focus-free period (`[[focus]]`, cvaq) when an idle number is wanted. The script is the same either
  way; only the start time differs. Cost: the first number is a busy one and is no baseline for idle.

**2. No fixed scenarios yet.** The sampler is passive: it watches, it drives nothing. The manifest
carries `scenario: passive` so that, when scenarios exist, runs are told apart without a schema
change. A scenario driver (tmux input, messages, spawn and stop) is a separate script that can run
alongside this one later. Not building it now is YAGNI and honours "no fixed scenarios yet".

**3. A log of what happened.** At the end the script writes `events.jsonl`: `bridle events --since
<start> --json` for the run window, plus, in the manifest, the daemon version, git sha of the daemon
and of the script, hostname, `uname`, and the pid of the daemon at each sample (a changed pid in the
CSV is a restart, visible without the events). If the daemon is down at the end, the export is
retried then recorded as failed in the manifest; the CSV still stands. Export is a plain read of an
existing command, so no new endpoint.

**4. Same script every time, and its form.**
- A. A standalone script in the repo, `scripts/bench/passive-sample.py` (stdlib Python, one
  long-lived process, constants for length and interval fixed in the file; no flags except an output
  directory). The manifest stores the script's blob hash (`git hash-object`) so a changed script is
  visible in the data. It refuses to start unless its own checkout is on `main`, clean, and not
  behind `origin/main`, which enforces point 5 mechanically.
- B. A `bridle bench` subcommand. Rejected: puts measuring code in the thing measured (a rebuild
  and daemon upgrade is needed to change it, which itself disturbs the run), adds a command nobody
  has asked for (KISS, YAGNI), and ties the benchmark to the daemon's release cadence.
- C. Fold into the phase 2 counters and skip the benchmark. Rejected for now: the counters do not
  exist, and the human wants a baseline before any enhancement lands (order section above).
- Recommend **A**. Python rather than shell: the CSV and parsing are simpler and robust, and shell
  rule `shell-zsh` word-splitting traps are avoided. Cost: depends on `python3` being present (the
  repo's test fake already does). Fallback if the human dislikes Python: the same script as bash.

**5. Commit and merge the script first.** Order: worker writes the script on a branch, the
manager merges to `main`, then the run starts from the main clone. The start refusal in 4A makes
skipping this impossible. The script may be written and merged before the human signs off; only the
run is gated.

**6. Shape and size of the output.** One run is one directory:
`manifest.json` (a few KB), `samples.csv` (120 rows x ~30 columns, about 30 KB), `events.jsonl`
(tens of KB, more on a busy hour), `sampler.log` (a few KB). Roughly 100 KB per run; weekly for
five years is about 25 MB. This is small enough for git. No compression or database needed.

**7. Where it lives, permanently.**
- A. In `main` (e.g. `docs/benchmarks/`). Rejected: clutters the integration branch, and cannot be
  purged without rewriting main's history.
- B. A dedicated orphan branch, `bridle/benchmarks`, pushed to origin, one directory per run named
  by UTC timestamp. Purging later is `git rm` on that branch or deleting it, and never touches main.
  It mirrors `bridle/state`.
- C. Only a dated folder on dalek in the bridle workspace parent, `<workspace parent>/benchmarks/
  <UTC timestamp>/`; copy to Dropbox later. Survives everything local, but has no history, review
  or off-machine copy until someone sets one up.
- D. Both: the script writes C continuously (first, outside any worktree, appended each sample, so
  a crash or landing loses nothing), and a separate `publish` step copies the finished directory to
  B with a commit and push. Publish is a second invocation of the same script so it can be redone.
- Recommend **D**. The folder is the safe write target; the branch is the reviewable, purgeable,
  off-machine record. Trade-off: two places to remember (state it in the docs). If the branch ever
  grows too big, C plus Dropbox remains and nothing else changes.

### Surviving the day (the human's "interrupt nothing")

- **No interruptions:** reads only (`ps` for a known pid list, `sysctl`, `vm_stat`, one `bridle
  status --json` per sample); about five short forks per 15 s; runs at low priority (`nice`). The
  manifest reports its own CPU. No pausing, holding or scheduling of agents.
- **Survives restarts and landings:** (1) never in a worktree: runs from the main clone and writes
  to the dalek folder (br-vt9k); (2) not a child of an agent or the daemon: started by the human or
  the PdM in its own tmux window (or `nohup`), so a daemon upgrade or an agent exit cannot take it
  down; landing (br-37r9) only removes agent worktrees and cannot touch it; (3) the daemon pid is
  looked up each sample from `daemon.json`; while the daemon is down the row has blank daemon
  columns, the sampler keeps going, and the gap is itself data; (4) rows are flushed on every
  sample, so a crash keeps what was written; a rerun is a new directory, not a resume.
- **Start gate:** the human signs off; then the run is started. Nothing in this design schedules it.

### Against the principles

- KISS/YAGNI: a single script, no new command, endpoint or daemon code; scenarios, Linux and
  counters are deferred with a named reason.
- Modularity: the sampler reads public surfaces (`ps`, `sysctl`, `bridle status`, `bridle events`)
  and couples to no daemon internals; it can be rewritten without a release.
- One name per action: no new `bridle` command (rejects B in point 4, avoiding a near-duplicate of
  `status`/`usage`/`events`).
- The user's side first: one command to start, one to publish, and a flat directory a human can
  open (`samples.csv` opens in any spreadsheet).
- Cost of not doing: no baseline, so the phase 2 counters cannot be shown cheap (the human's order).

### Recommendation

Passive 30-minute sampler (4A), run live now with load context recorded (1B), no scenarios yet (2),
event-log export plus manifest (3), script merged first and self-checking (5), written first to a
dated folder in the workspace parent and published to an orphan branch `bridle/benchmarks` (7D),
about 100 KB per run (6), run detached from agents and worktrees so restarts and landings cannot
touch it. Accepted trade-offs: the first number is a busy number; forks per minute stays unmeasured
until the counters exist; Python is required; two storage locations to document.

### Questions for the human

1. Python script, or bash? (Recommendation: Python.)
2. Is `bridle/benchmarks` as a branch name right, and is pushing it to origin approved?
3. Should a quiet-window (idle) run be asked for separately, after the first live one?

## Sign-off (the human, 2026-10-10 ~10:15 AM ET)

The human approved the design above (Q1 Python, Q2 `bridle/benchmarks`, Q3 an idle run, Q4 the
orchestrator starts it) and the sequence: a live passive run, an informal check that it worked,
the idle run tonight ~4:00 AM ET (15-20 min prep, 30 min run), then an agent's report on both
for Sunday morning. Verbatim (to the aide, relayed as m-9234), the benchmark parts:

> Okay, I'm reviewing the design, and I think that looks great. I agree. I think, on all these points, yes, I think Python is the right choice here. It's easy to write, easy to run. I'm very familiar with it and very comfortable with it. Building big in Bash is a pain, so yeah, definitely agree on that. If we need some kind of Bash script to kick it off or something, that's totally great. That's fine, but the core of everything should be in Python. I like `bridle/benchmarks`. Great name. Definitely, we should do an idle run.
>
> Let's just try this again. Let's get that plan. If the passive run is after we do one pass run, we'll check the results and see what happens. Do a kind of postmortem, not a formal one, and see if it worked. Assuming it worked okay, then we'll schedule an idle run for tonight. Again, around 4:00 am is good. We should plan ahead to get some work paused or landed before that starts. Probably needs 15, 20 minutes to barely get in shape, in place, and do a 30-minute run where nothing's happening on the system. After 30 minutes is plenty.
>
> We'll probably just get a baseline of the exact same thing for most of that 30 minutes, and then end it and save that off. If that all goes off without a hitch, then have an agent investigate both and have a report ready for Sunday morning that analyzes and summarizes what was in there and what happened.
>
> I'll probably like some of this you can have in the dashboard, the bridle-ui, maybe. I also work a lot, and I'm a data scientist as well, so having this in a Jupyter notebook in the future (like some ready-made Python functions I can call from the notebook to load up all the benchmark data and create some basic charts from it) would be something. That's a future thing. Don't do that right now, but add that to this theme.
>
> And then, after this or during this, I want the messaging work to really get done. What we've got planned so far, the scheduled messages, is a real high priority for me. Assuming the setup for the WSL2 work is done, assuming the machine work is done, adding scheduled messages is the next priority.
