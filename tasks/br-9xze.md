+++
id = "br-9xze"
title = "Scheduled messages, first slice: an agent schedules a message to itself (one-time or recurring), bridle schedule add/list/rm"
kind = "feature"
state = "integrated"
created_at = "2026-10-08T14:28:58.540Z"
updated_at = "2026-10-09T04:54:31.554264Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:orchestrator@nuc",
]
branch = "bridle/schedmsg"
commit = "dc189a801d0e3399814929083bee34407c7a6c7e"
summary = 'Scheduled messages slice 1. New `schedules` table (SCHEMA_V23); daemon `schedule.rs` (own 5-field cron parser, DST-aware via chrono-tz, which is a new dependency because nothing in Cargo.lock handled IANA zones; `fire_due` with an injected clock); a 15 s loop plus one run at start-up sends due schedules from `system` as "Scheduled <id> (set by <creator>): ..." through the normal message path; a missed once fires late, a missed cron fires once for its latest occurrence, both with a "(due <local>, sent late)" note when over 2 minutes late. Routes /v1/schedules (POST, GET ?all, DELETE /{id}), client methods, `bridle schedule add|list|rm`, `[schedule] timezone` config, events schedule.fired / schedule.missed_fired. Docs: api.md, cli.md, storage.md, roles-and-config.md, daemon.md "Scheduled messages", CHANGELOG. Caveats: externals (orchestrator, advisor) are refused, per decision 6 (strictly agents and the human); a send to a vanished target is logged and the schedule moves on; targets are principals on the same daemon (no outbox hop).'
parent = "br-yfv5"
+++

Tickets: docs/tickets/open/scheduled-messages-an-agent-or-the-human-schedules-a-message-hrcn.md (the human's ask, verbatim) and docs/tickets/open/one-scheduler-for-timed-actions-scheduled-messages-hrcn-nigh-yfv5.md (shape). Read both. The human's approval (2026-10-08, kept below) is for ONLY this slice: per-project scheduled MESSAGES. Not cbbn (nightly restarts), 3nyk (maintenance windows), cy2v (machine-wide).
Approval, verbatim: "an agent wants to set itself a wake-up ... They set their wake-up timer to the maximum and then rely on scheduled message sends to wake themselves up, and that gives them a reminder of what the purpose of the wake-up is."

Slice 1 = the mechanism. (Role priming is a separate task, br-<slice 2>; do not edit workflow/base/roles here.)

Decisions already made (do not re-open; if one fails in practice, comment on the task):
1. Storage: a new table `schedules` in the daemon DB (crates/bridle-daemon store; add a migration step the way the store adds tables, see docs/design/storage.md): id (`sc-` + 4 base36 chars like task ids), created_by (principal), target (a principal exactly as `bridle send` takes it, e.g. `external:orchestrator`, `agent:notes-1`), body, kind (`once` | `cron`), at_utc (once), cron (cron), tz (IANA name; default `America/New_York` from an optional `[schedule] timezone` in the daemon config), next_fire_at (UTC), last_fired_at, state (`active` | `done`), created_at. A fired `once` becomes `done`; a `cron` stays `active` and gets its next_fire_at.
2. Cron: standard 5-field expressions evaluated in the schedule's tz with DST handled (a nonexistent local time is skipped that day; an ambiguous one fires once, at the first occurrence). Use a crate already in Cargo.lock if one fits (check chrono / chrono-tz / jiff / cron); add one only if none does and say why in the done note. `unsafe` stays forbidden.
3. Firing: a daemon loop wakes every 15 s (a const), finds `active` rows with next_fire_at <= now, and sends the message to the target through the normal message path (so it queues, wakes the agent like any message, and across-daemon targets use the outbox). The message is from `system` (the existing system sender) with the body prefixed `Scheduled <id> (set by <created_by>): ` so the recipient sees why it woke.
4. Missed firings (daemon down, machine asleep): on start-up and on each tick, a `once` whose time passed fires once, now; a `cron` fires ONCE for the latest missed occurrence (never a burst), then moves to the next future occurrence. Fired-late messages say so: append ` (due <local time>, sent late)` when more than 2 minutes late.
5. CLI (crates/bridle): `bridle schedule add --to <principal> (--at <time> | --cron "<expr>") [--tz <IANA>] [--message <text> | --message-file <path|->]` where `--to` defaults to yourself (an agent) and `--at` takes RFC 3339 or `YYYY-MM-DD HH:MM` in the tz; prints the id and the next fire time in the human's zone. `bridle schedule list [--all]` (an agent sees its own; the human sees all; `--json`). `bridle schedule rm <id>`. Reject a cron that never fires or a past `--at` with a clear message.
6. Auth: an agent may add a schedule only with itself as the target, and list/rm only its own. The human may add for any target and list/rm any. Everyone else refused. API: routes under /v1/schedules (types in crates/bridle-api/src/types.rs; client methods in the client module); update the wire types, daemon and CLI together.
7. Events: `schedule.fired` and `schedule.missed_fired` recorded with the id and target; none per tick.
Docs: new section in docs/design/agent-host/api.md (routes), docs/design/cli.md (commands), docs/design/storage.md (table), a short `docs/design/scheduler.md` is NOT needed: put the decisions above into daemon.md under a heading "Scheduled messages". CHANGELOG.
Migration: the new table is created by the store's normal schema upgrade on daemon start; no project files change; the commands reach agents on daemon upgrade.
Acceptance: just check passes; tests with an injected clock (tokio paused time or a clock fn): once fires and goes done; cron fires and reschedules; DST spring-forward and fall-back cases; missed once fires late with the note; missed cron fires once not many; agent cannot target another agent, human can; list scoping; rm; a restart (reopen the store) keeps schedules.
Model: Sonnet. Right-size: if this passes about 200K tokens of context, land the store + firing loop + API first and file the CLI as its own task via `bridle task new --from br-9xze`.
Out of scope: quiet-hours handling (scheduled messages fire during them), cross-machine schedules (a schedule lives on one project's daemon), nightly restarts, maintenance windows.

## Thread

### note · agent:pm-1 · 2026-10-08T14:29:41.499Z
split off br-g5y2: Scheduled messages slice 2: role priming: wait at the maximum timeout and schedule a message for timed wake-ups (hrcn)

### note · agent:schedmsg · 2026-10-09T02:22:40.617Z
done: scheduled messages slice 1 (store, firing loop, /v1/schedules, bridle schedule add|list|rm, docs, CHANGELOG, summary written); just check exit 0, 1362 tests passed; 0e966960. Notes: added chrono-tz (nothing in Cargo.lock handled IANA zones); externals like the orchestrator are refused per decision 6 (agents and the human only), which slice 2 may want to revisit; targets are principals on the same daemon.

### note · agent:manager-2 · 2026-10-09T02:30:19.850Z
manager-2: land hit a merge conflict in crates/bridle/src/commands/mod.rs (br-rcvb added 'report' there). Merge main, resolve, run just check once, report the tip.

### note · agent:schedmsg · 2026-10-09T03:51:16.573Z
merged main, tip 4dafeaf3; full check flaked on load-sensitive daemon timing tests (different one each run, pass alone), see thread

### note · agent:schedmsg · 2026-10-09T03:51:18.857Z
Merged main (conflict in commands/mod.rs: kept both report and schedule mods), tip 4dafeaf3. Full just check under host load 50-60 failed on a different timing test each run (governor_test working_agent_is_notified_then_stopped..., then upgrade_test a_daemon_raised_wake...); the whole upgrade_test suite passes alone (12/12). 1146 of 1147 run passed; not a clean full green, so re-run check on a quieter host for certainty.

### note · agent:schedmsg · 2026-10-09T04:28:05.573Z
Re-ran just check on 4dafeaf3: exit 0, 1366 tests run, 1366 passed (5 skipped). The earlier failures were load flakes.

### note · agent:schedmsg · 2026-10-09T04:28:11.026Z
done: just check exit 0, 1366 passed, on 4dafeaf36065becd2fc5a7b1ca2f8a66103ed1eb (main merged)

### note · agent:manager-2 · 2026-10-09T04:54:31.554Z
integrated: dc189a801d0e3399814929083bee34407c7a6c7e (branch bridle/schedmsg)
