---
id: k6b3
title: A worker's test ran the live orchestrator's launcher
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [the-orchestrator-stays-running-fx7x, orchestrator-pane-full-of-escape-codes-csfe]
closed: 2026-09-30T05:12:44Z
---

## What happened

On 2026-09-30, between 03:25 and 03:26 UTC, worker `nuc-scripts` (br-f02f, NUC B1) tested its
changes to `scripts/claude-orchestrator` by running the script for real, several times. Each run:

- started a real `claude` with `--remote-control orch`, the live orchestrator's session name. One
  test prompt (the script's own path) reached the live orchestrator as if the human had typed it,
  twice;
- overwrote `~/.bridle/orchestrator.pid` and, through the SessionStart hook,
  `~/.bridle/orchestrator.session`. The daemon then supervised a dead test session: a false
  "orchestrator is not running" note to the human (m-2319), and context readings from the wrong
  transcript;
- appended a line to `~/.bridle/orchestrator.exits`.

The orchestrator told the worker to stop (m-2320). The human restored both files by hand, because
the orchestrator's auto mode refused to write them.

## Why it could happen

The launcher writes to `$BRIDLE_HOME` (default `~/.bridle`) and names its Remote Control session
from a fixed prefix. A worker's shell has the same home, so nothing separates a test run from the
real one. The worker rule "don't spawn real `claude`" (CLAUDE.md) covers tests in the suite, and
the worker read it that way, not as covering a hand-run script.

## Options

- Workers run with `BRIDLE_HOME` set to a directory of their own (the daemon sets it at spawn), so
  a launcher run by a worker can't touch the live files. Cheapest, and it covers other scripts too.
- The launcher refuses to start when it's run by a bridle agent (e.g. `BRIDLE_AGENT_ID` is set),
  unless it's given a test flag.
- A worker rule: never run `claude-orchestrator` or `claude-advisor` for real; test with `bash -n`,
  or with `claude` stubbed on `PATH` and a temporary `BRIDLE_HOME`.

Recommendation: the first two. A rule alone already failed once.
