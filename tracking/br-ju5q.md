+++
id = "br-ju5q"
title = "Benchmark report: analyse the live and idle runs, ready Sunday 2026-10-11 morning"
kind = "research"
state = "integrated"
created_at = "2026-10-10T14:20:12.659Z"
updated_at = "2026-10-12T00:52:01.752956Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:aide",
    "agent:wju5q",
]
size = "S"
branch = "bridle/wju5q"
commit = "88382f3406626218bbf897820337d5a8af2dbfaa"
summary = "Baseline report added to the v6kr ticket under '## Baseline report (2026-10-11)'; analysis script scripts/bench/analyse-runs.py. Daemon is cheap (~80-100 MB, ~4% of a core); agents dominate (~4.3 GB RSS idle). Idle run's daemon CPU was inflated by a ~105 s burst around the disk.checked event. Other oddities and next counters/scenarios listed in the report."
ticket = "ju5q"
parent = "br-v6kr"
+++

docs/tickets/open/benchmark-report-analyse-the-live-and-idle-runs-ready-sunday-ju5q.md

Ticket: docs/tickets/open/a-system-architect-role-and-measuring-bridle-s-own-resource-v6kr.md. The human, 2026-10-10: 'have an agent investigate both and have a report ready for Sunday morning that analyzes and summarizes what was in there and what happened.' Read both runs from bridle/benchmarks (samples.csv, events.jsonl, manifest.json). Report: what bridle's daemon(s) and agents cost the machine idle vs. busy (CPU deltas, RSS, memory pressure, compressed, swap), what happened during each run (from the events), the sampler's own cost, gaps and anything odd, and what it suggests next (counters, scenarios). Plain and short, ASCII. Put it in the ticket under '## Baseline report (2026-10-11)' or as a file the design names; commit and merge. Python (stdlib or what the repo already has) for any analysis; commit the analysis script too. Start once the idle run is published (~4:35 AM ET).

## Thread

### note · agent:wju5q · 2026-10-12T00:51:26.689Z
done: baseline report in v6kr ticket + scripts/bench/analyse-runs.py; just check exit 0, 1550 tests (1538 last full), commit 2ff4b16d7b8c002c8108249f19fa3c7ee5324379. Headline: idle daemon burst ~105s CPU at disk.checked.

### note · agent:wju5q · 2026-10-12T00:51:29.821Z
built: 2ff4b16d7b8c002c8108249f19fa3c7ee5324379 (branch bridle/wju5q)

### note · agent:manager-2 · 2026-10-12T00:51:32.391Z
integrated: 88382f3406626218bbf897820337d5a8af2dbfaa (branch bridle/wju5q)

### note · agent:manager-2 · 2026-10-12T00:52:01.752Z
cleanup: removed agent wju5q, branch bridle/wju5q
