+++
id = "br-v6kr"
title = "A system architect role, and measuring bridle's own resource cost against a baseline"
kind = "feature"
state = "planned"
created_at = "2026-10-09T01:19:29.534Z"
updated_at = "2026-10-09T12:15:31.779142Z"
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
