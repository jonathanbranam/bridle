# Benchmarks: the passive resource sampler

> **Status (checked 2026-10-10):** Built: `scripts/bench/passive-sample.py` (br-57ec). No run has been made; the first live run is gated on the human's sign-off. Design and rejected options: ticket v6kr, "Design options".

A repeatable 30-minute reading of what bridle costs the machine, kept permanently. The sampler is
passive (`scenario: passive`): it watches and drives nothing. A scenario driver, if wanted, is a
separate script that runs alongside it.

## Running it

```
scripts/bench/passive-sample.py [--out BASE]    # sample, 30 min, every 15 s
scripts/bench/passive-sample.py publish RUN_DIR # copy a finished run to the bridle/benchmarks branch
just bench-test                                 # unit tests (in `just check`); they never sample
```

- Run it from the main clone, in its own tmux window, never from an agent or a worktree (a landing
  removed the worktree and the first baseline with it, br-vt9k). It refuses to start unless its
  checkout is `main`, clean and not behind `origin/main`, so the script is always the merged one.
- Length and interval are constants in the file; the only option is the output base directory.
  Default: `<workspace parent>/benchmarks/` (beside the workspace, e.g. `/Volumes/Data/work/benchmarks/`).
- It runs at `nice 10`, makes about five short reads per sample (`ps`, `sysctl`, `vm_stat`, one
  `bridle status --json`) and never writes to the daemon.
- SIGINT/SIGTERM end the run early and still write the manifest (`status: interrupted`). A rerun is a
  new directory, never a resume.

## Output

One directory per run, named by UTC start time (`20261010T150000Z`), about 100 KB:

| File | Holds |
|---|---|
| `samples.csv` | 120 rows, flushed each sample. CPU is cumulative seconds (take deltas). Daemon columns are blank while the daemon is down; `daemon_pid` changing is a restart. `children_*` are the daemon's direct non-`claude` children; `claude_*` are all `claude` processes; kept apart so a `ps`-style child shows as the daemon's cost. Memory: `pressure_level`, `compressed_kb`, `swap_used_mb` (macOS only; never top's used/free). |
| `manifest.json` | scenario, constants, start and end, hostname, `uname`, daemon version, git sha and blob hash of the script, daemon pids seen, load context (`working_mean`, `working_max`, `class` = `idle` if no sample had a working agent, else `busy`), the sampler's own CPU seconds, events export status, and `spawns` (count of `agent.*` events). |
| `events.jsonl` | the event log for the run window (`bridle events --since <seq at start>`), one event per line. Exported because the log is pruned at 30 days. If the export fails it is retried, then recorded as failed; the CSV stands. |
| `sampler.log` | the sampler's own log. |

Compare only like with like: an `idle` run with an `idle` run. Limit: a 15 s sampler cannot see forks
per minute; `spawns` from the event log is all it claims, until in-daemon counters exist.

## Where it lives

Two places, by design:

1. The dated folder in the workspace parent, written continuously and outside any worktree: the safe
   write target. Copy it to Dropbox or elsewhere later for retention.
2. The orphan branch `bridle/benchmarks` (one directory per run), pushed to origin by `publish`: the
   reviewable, off-machine record. Purge with `git rm` on that branch or delete it; `main` is never
   touched. `publish` can be redone (it replaces the run's directory and commits again).

Deferred (YAGNI): Linux memory measures, scenarios, a `bridle bench` command.
