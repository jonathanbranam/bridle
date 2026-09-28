+++
id = "br-7ab7"
title = "bridle budget: local times, the applied budget and the schedule (c424)"
kind = "chore"
state = "planned"
created_at = "2026-09-28T17:38:41.415Z"
updated_at = "2026-09-28T17:53:41.780195Z"
+++

ticket: docs/questions/open/budget-times-in-local-time-c424.md
original id: c424

Two parts, both settled by the human (2026-09-28), low priority but no longer Haiku-sized
now that the second part landed (small-to-medium, Sonnet).

**1. Local time, not UTC.** `bridle budget` prints reset/hold times as UTC today (e.g.
"resets 2026-09-28 19:19 UTC"), hardcoded as `"%Y-%m-%d %H:%M UTC"` in
crates/bridle/src/commands.rs (lines 1006, 1275, 1283, 1290 @ 65a64c4 -- line numbers will
have drifted, grep for the format string). The human: "fine resolve that question and use
the local machine timezone" (not fixed Eastern -- the machine's own local tz, via chrono,
already a dependency). Note this is a display-only change to `bridle budget`/`bridle
status`'s own output; it's a narrower case than the standing rule in
workflow/base/rules/human-timezone.md (which is about times shown *to the human*
generally, Eastern by convention) -- here the human explicitly chose local-machine-tz
instead for this specific command, so follow that, not the base rule's Eastern default.

**2. Show the applied budget and the schedule.** The human, verbatim: "there should be a
way to see the current applied budget and the next change, it's span, and settings; also
an option to print the entire schedule out (make it easy so the user doesn't have to scan
the various config files)". Concretely:
- `bridle budget` (no flags) additionally shows which schedule entry applies right now
  (its name, its span, and its `hold_at`/`wind_down_at`/`stop_at` thresholds), and when
  the next change happens and to what entry -- in local time, per part 1.
- A new `--schedule` flag prints the whole resolved schedule (every entry, its span and
  thresholds), so the human doesn't have to read `~/.bridle/config.toml` and the project
  config by hand to work out what applies when.
- Find where the schedule is currently parsed/resolved (likely
  crates/bridle-daemon/src/config.rs or a budget-governor module -- search for
  `hold_at`/`wind_down_at`/`stop_at`) and reuse that resolution rather than re-parsing
  config in the CLI; if the CLI doesn't have access to the resolved schedule today (e.g.
  it's daemon-internal state), add a read path (API endpoint or existing `bridle budget`
  response field) rather than duplicating the schedule logic client-side.

Acceptance: just check passes; `bridle budget` output uses local machine time everywhere
it prints a time; `bridle budget` shows the current schedule entry and the next change;
`bridle budget --schedule` prints the full resolved schedule; `--json` output stays
UTC/ISO8601 (machine-readable, unaffected by the local-time display change -- confirm
this is already true today and keep it that way); docs/design/cli.md updated for the new
flag and the (now local-time) output shape.

Out of scope: changing how times are recorded internally (bridle, git, tickets, logs stay
UTC per workflow/base/rules/human-timezone.md); anything about the schedule's own
semantics or config format (this task only surfaces the existing schedule, doesn't change
it).

Model: Sonnet (touches schedule resolution plus CLI output, not purely mechanical).

## Thread

### note · external:orchestrator · 2026-09-28T17:43:03.392Z
The human widened c424 (2b0b194): besides local times, bridle budget shows the schedule entry that applies now (name, span, hold/wind-down/stop thresholds) and the next change; plus an option to print the whole resolved schedule. See the ticket's last section.
