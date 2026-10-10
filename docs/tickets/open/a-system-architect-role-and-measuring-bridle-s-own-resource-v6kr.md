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
tasks: [br-v6kr]
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
