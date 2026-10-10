+++
id = "br-v6kr"
title = "A system architect role, and measuring bridle's own resource cost against a baseline"
kind = "feature"
state = "planned"
created_at = "2026-10-09T01:19:29.534Z"
updated_at = "2026-10-10T02:07:28.393419Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
ticket = "v6kr"
+++

Ticket: docs/tickets/open/a-system-architect-role-and-measuring-bridle-s-own-resource-v6kr.md (read all of it: the human's verbatim words and the order they set). This task is PHASE 1 ONLY, the human's first step: "establish the baseline before doing any of these additional enhancements". The counters (phase 2), the weekly benchmark scenarios and the architect role are NOT in this task; they wait for the baseline and the human's review (the human wants to see the scenario list before it is built).

Do, in two parts:
1. Baseline measurement (a one-off, not a feature). Write scripts/baseline-sample.sh (bash, no new tools; plain `ps -o` and `uptime`/sysctl, no unsafe, nothing resident) that, once a minute for a given duration, appends one CSV line to a file given as argument: UTC time, 1-minute load, for each running bridle daemon (find them from `bridle daemon list`/the registry command if one exists, else from the pid files under each project's .bridle/): pid, cumulative CPU seconds (ps -o time), RSS KB, number of child processes (direct children by ppid), and the count of all processes on the machine. Start it yourself in the background (run_in_background) for 12 hours, idle machine assumed (the human wants it overnight; if the machine is busy with other agents at the time, the sample still records that: add the count of running agents from `bridle agents --json`). Do not use pkill/killall/pgrep; stop it by the pid of the background job you started (TaskStop).
   Write the results (not the raw CSV: a summary table of per-daemon CPU seconds per hour, RSS min/avg/max, child-process count min/avg/max, load min/avg/max, split by "no agents running" vs "agents running") to docs/context/baseline-2026-10.md, with the machine (dalek, cores, macOS version), the daemon commit, and the exact command. The raw CSV goes next to it (docs/context/baseline-2026-10.csv) if under 200 KB, else keep a downsampled hourly version.
2. A proposal for the next steps, for the human to review, in docs/design/benchmark.md (status line "planned"): (a) the scenario list for the repeatable benchmark with each scenario's steps, duration (10-20 min each, sequential, same machine only), and what is measured, using the human's examples: simulated input to an interactive session through tmux; messages between agents, between daemons and between projects; starting a background worker, having it do something and stopping it, a few times in different parts of the system; (b) the phase-2 counters: a line per minute written to a log file by the daemon, from numbers it already has (getrusage self and children, processes started counter incremented at the existing spawn sites, RSS, load), with an estimate of its own cost, no fork, no endpoint; (c) how the baseline gets compared; (d) the architect role in one paragraph: on demand, not standing, reads this and runs the check. Mark every open question as a Q.

Files: scripts/baseline-sample.sh (new), docs/context/baseline-2026-10.md and .csv (new), docs/design/benchmark.md (new), docs/README.md index lines. No Rust changes.

Acceptance: the sampler produced at least 10 hours of lines (or say why not), the summary is in the doc, benchmark.md exists with the scenario list. just check passes (no code change, but run it once). Ask the human nothing: send the proposal to the orchestrator for review via the task thread.

Model: Sonnet. Migration: none. Out of scope: any daemon code, counters, tokio-console/profiling, the architect role file, the weekly schedule, inter-machine scenarios, fixing anything the baseline shows (file a ticket instead).

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:40.435Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T12:15:31.779Z
PdM (advisor product-manager): whoever designs or runs the phase 2 baseline: measure memory as the ticket's section 'Memory: which measure' says (macOS: pressure level, compressed, swap; bridle's own: footprint/RSS; Linux: MemAvailable and PSI), not top's or sysinfo's used/free, which counts file cache as used on macOS. The human's question, 2026-10-09 ~8:20 AM ET, is quoted there.

### note · external:orchestrator · 2026-10-09T22:00:40.978Z
Orchestrator: the human (2026-10-09 ~6:05 PM ET): "put it at like 4am". Start the baseline at 4:00 AM ET (08:00Z, 2026-10-10), not 9 PM. The orchestrator sends manager-2 the go then; don't start it before. While it runs, start no other worker.

### note · external:orchestrator · 2026-10-09T22:00:41.153Z
From orchestrator: br-v6kr: comment added (start moved to 4:00 AM ET / 08:00Z by the human; hold until my go)

### note · external:orchestrator · 2026-10-10T02:05:37.694Z
orchestrator: the human, 2026-10-09 ~10:25 PM ET: "please carry on over night ... yes, if possible, run the benchmark tonight. if not, the machine setup theme is the priority. follow the PdM's guidance." Plan: go at 4:00 AM ET (08:00Z) once in-flight workers (br-88d4, br-751e) have finished their checks; no new worker starts after that until the baseline ends. If they can't be clear by ~5 AM, machine setup keeps the slots and the baseline moves (PdM decides).

### note · external:orchestrator · 2026-10-10T02:05:37.784Z
From orchestrator: br-v6kr comment added (the human wants the baseline tonight if possible). Keep going with br-88d4 then br-751e now, but don't start a worker after ~3:00 AM ET that can't finish its check by 4:00 AM. At 4 AM I'll send the go; nothing new starts while it runs.

### note · external:advisor/product-manager · 2026-10-10T02:05:48.757Z
advisor (product-manager): agreed. Order until 3 AM: br-88d4, br-751e, then br-fpde if it can finish its check by 4 AM. If the slots aren't clear by ~5 AM, the baseline moves to tonight (10-10) and machine setup keeps the slots.

### note · external:advisor/product-manager · 2026-10-10T02:07:13.462Z
advisor (product-manager): the human, ~10:10 PM ET: "benchmark can wait if necessary so we get WSL2 online tomorrow." Machine setup has the slots all night; the baseline runs only if br-88d4, br-751e and br-fpde are done and checked first.

### note · external:orchestrator · 2026-10-10T02:07:28.303Z
orchestrator: superseded by the human via advisor (product-manager), ~10:10 PM ET: "benchmark can wait if necessary so we get WSL2 online tomorrow." Machine setup (br-88d4, br-751e, br-fpde, then br-hdbj) uses every slot all night; the baseline runs at/after 4 AM only if they are all done and checked, otherwise it moves.

### note · external:orchestrator · 2026-10-10T02:07:28.393Z
From orchestrator: correction to my last: the human (via the PdM) puts machine setup first tonight. Use both slots all night for br-88d4, br-751e, br-fpde, then br-hdbj (branch + just check; merges wait for the human). No 3 AM cutoff. The baseline starts only if those are all done and checked; I'll check at 4 AM. No low-priority or designer work. I stopped the three idle doc reviewers.
