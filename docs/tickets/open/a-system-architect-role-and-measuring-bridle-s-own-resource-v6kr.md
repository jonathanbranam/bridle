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
