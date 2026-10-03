+++
id = "br-7ab7"
title = "bridle budget: local times, the applied budget and the schedule (c424)"
kind = "chore"
state = "dropped"
created_at = "2026-09-28T17:38:41.415Z"
updated_at = "2026-09-28T23:36:34.769318Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
+++

ticket: docs/questions/open/budget-times-in-local-time-c424.md
original id: c424
merged in: docs/questions/open/bridle-budget-shows-what-applies-h8tq.md (h8tq) -- the human
asked for h8tq with "yes, file that issue to make the budget accurate" and it overlaps this
task's part 2 enough that the ticket itself says "do them together"; one task, not two.
see also: kv7d (allowed_warning forces wind-down even under a raised override -- open,
awaiting the human's answer, not yet scheduled)

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

**2. Show the applied budget and the schedule, accurately (c424 + h8tq).** The human,
verbatim: "there should be a way to see the current applied budget and the next change,
it's span, and settings; also an option to print the entire schedule out (make it easy so
the user doesn't have to scan the various config files)". Then, after a `burst` override
looked wrong (h8tq): "yes, file that issue to make the budget accurate". Concretely, seen
broken @ d45523b:
- `GET /v1/budget` returns `thresholds: cfg.to_wire()` (crates/bridle-daemon/src/server.rs:370),
  the plain config, not what `effective_five_hour_thresholds` actually applies (an override
  or schedule period). `bridle budget` must show the *applied* thresholds and where they
  come from: override name, schedule period name, or default -- not the static config.
- `bridle budget` (no flags) additionally shows which schedule entry (or override) applies
  right now (its name, span, and `hold_at`/`wind_down_at`/`stop_at`), and when the next
  change happens and to what -- in local time, per part 1.
- A new `--schedule` flag prints the whole resolved schedule (every entry, its span and
  thresholds), so the human doesn't have to read `~/.bridle/config.toml` and the project
  config by hand.
- `stale: rl.is_none()` (server.rs) only means "no reading"; a reading hours old shows as
  fresh. Show each reading's age, and mark it stale by age (`max_staleness`), not only when
  missing.
- `bridle budget`'s text should show a rate limit's `status` (e.g. `allowed_warning`,
  `rejected`) when it isn't plain `allowed`, since that's what can drive a state change.
- Find where the schedule is currently parsed/resolved (likely
  crates/bridle-daemon/src/config.rs or a budget-governor module -- search for
  `hold_at`/`wind_down_at`/`stop_at`) and reuse that resolution rather than re-parsing
  config in the CLI; if the CLI doesn't have access to the resolved schedule today (e.g.
  it's daemon-internal state), add a read path (API endpoint or existing `bridle budget`
  response field) rather than duplicating the schedule logic client-side.

Note on kv7d (open, unresolved): `allowed_warning` currently forces wind-down even under a
raised override, which can make the reported state flip between `normal` and
`winding_down` for reasons this task's "show why" output should surface honestly (state,
threshold, and status that produced it) even before kv7d is resolved -- don't block this
task on kv7d, but don't paper over the contradiction either; show it accurately, in the
same live-and-let-the-human-see spirit as h8tq's ask. If kv7d resolves first, this task's
"why" text may need a small follow-up touch, not a redo.

Acceptance: just check passes; `bridle budget` output uses local machine time everywhere
it prints a time; `bridle budget` shows the currently-applied thresholds (override or
schedule period, not raw config) and where they come from, plus the next scheduled change;
`bridle budget --schedule` prints the full resolved schedule; each reading shows its age
and is marked stale by age via `max_staleness`, not only when missing; non-`allowed`
statuses are shown; `--json` output stays UTC/ISO8601 (machine-readable, unaffected by the
local-time display change -- confirm this is already true today and keep it that way);
docs/design/cli.md updated for the new flag and the (now local-time, applied-not-static)
output shape.

Out of scope: changing how times are recorded internally (bridle, git, tickets, logs stay
UTC per workflow/base/rules/human-timezone.md); kv7d's own fix (whether allowed_warning
should force wind-down at all); a probe-when-idle usage reading (h8tq's optional idea,
follow-up if this task's age/staleness display isn't enough on its own).

Model: Sonnet (touches schedule/threshold resolution, server response shape, and CLI
output together).

## Thread

### note · external:orchestrator · 2026-09-28T17:43:03.392Z
The human widened c424 (2b0b194): besides local times, bridle budget shows the schedule entry that applies now (name, span, hold/wind-down/stop thresholds) and the next change; plus an option to print the whole resolved schedule. See the ticket's last section.

### note · agent:pm-1 · 2026-09-28T23:36:34.769Z
dropped: Merged to main.
