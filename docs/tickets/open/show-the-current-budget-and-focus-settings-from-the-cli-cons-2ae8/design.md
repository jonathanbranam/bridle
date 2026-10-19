# Design

Proposal: bare form for both, `--schedule` and `--json` on both, no `show` verb. This design
decides where focus is read and what the layout is.

## Problem

- `FocusAction` (`crates/bridle/src/cli.rs`) has only `Gate` and `Reply`, and the subcommand is
  required, so bare `bridle focus` is a usage error. Nothing prints the `[[focus]]` periods.
- Budget's read is already there but was built alone, so focus has nothing to match. The text
  layout and `--json` of the two must line up (same labels, same JSON key names).
- The proposal says focus has no state to override. It does: `~/.bridle/focus-override.toml`
  (hand-written, capped at 2 h, delayed start; roles-and-config.md, "Override"). The read must
  show it, as `bridle status` already does for one line (`status_line` in
  `crates/bridle/src/focus.rs`).

## What exists today

- Budget: `budget()` in `crates/bridle/src/commands/usage.rs` calls `GET /v1/budget` and prints
  `BudgetStatus` (`bridle-api/src/types.rs`): `state`, `why` lines, `hold` (with end), `override`
  (with end), applied thresholds with source and `span`, `next change`, windows, caps.
  `--schedule` prints `print_schedule`: one line per period. `--json` prints `BudgetStatus`
  as is. So budget already shows the configured periods, the period in effect and when it
  changes, and any hold or override. Time text is local (`local_time`).
- Focus: config parsing and the maths live in `bridle-daemon` (`config::focus_periods`,
  `FocusPeriod::matches`, `focus_end`, `focus::locked_period`, `focus::read_override`), and the
  `bridle` crate already calls them directly, with no daemon (`gate`, `status_line`,
  `refuse_advisor_if_locked`). The hook must never fail, so it cannot depend on a daemon.
- Budget `GovernorState` has no focus analogue; the focus analogue is the period `mode`
  (`quiet`, `locked`) and "no period now".

## Options

### Option 1: Read focus in the CLI, mirror budget's layout

- User view: bare `bridle focus` prints, in budget's style (label, then detail):

  ```
  now      quiet: work (mon,tue,wed,thu,fri 08:00-18:00)
    ends     2026-10-18 18:00 EDT
    next change 2026-10-18 18:00 -> none
  override active until 2026-10-18 17:00 EDT: deploy is broken
  ```

  or `now      none` outside every period. `--schedule` prints one line per period:
  `work  mon,tue,... 08:00-18:00  quiet`. `--json` prints `{current, next_change, override,
  schedule}`, where `schedule[]` items are `{name, span: {days,start,end}, mode}` (same
  `ScheduleSpan` as budget) and times are UTC. With no `[[focus]]`: `no [[focus]] periods;
  focus hours are off`.
- Internals: make `FocusArgs.action` optional and add `--schedule`. One new function in
  `crates/bridle/src/focus.rs` builds a `FocusStatus` from `focus_periods`, `focus_end`,
  `read_override` (pure, takes `home` and `now`, unit-testable). `FocusStatus` goes in
  `bridle-api/src/types.rs` next to `ScheduleSpan` and reuses `format_span`/`local_time`.
  Budget text and JSON stay as they are; the only budget change is to rename nothing and make
  sure the shared labels (`override`, `next change`, `span`) are the same strings.
- Cost: small; no daemon change, no new endpoint. Agents get the answer by running the command,
  not by reading the file.
- Shortfall: JSON is "one layout" only by shared names and shared `ScheduleSpan`, not one type:
  budget's JSON has governor fields focus has no use for. Works per machine config, which is
  what focus is anyway.

### Option 2: A daemon endpoint `GET /v1/focus`

- User view: same as option 1.
- Internals: a route, client method and handler that does what option 1's function does, and
  the CLI prints the reply.
- Cost: wire type plus route plus client plus daemon code, all for a read the CLI can already
  do locally; and a daemon is needed. The daemon is per project while `[[focus]]` is machine
  config, so the answer would depend on which daemon is asked.
- Shortfall: breaks the "local, never fails" property focus has everywhere else, and fails when
  the daemon is down, exactly when the human wants to check their hours.

### Option 3: One shared status type for both

- User view: budget and focus print and serialize through one `PeriodStatus` type
  (`current`, `next_change`, `override`, `schedule`).
- Internals: reshape `BudgetStatus` to wrap it; budget's governor fields stay beside.
- Cost: changes budget's JSON (the proposal's risk) or needs a compatibility layer, for a
  consistency the human can already read in the labels.
- Shortfall: couples the two by a type for a gain of "same field names", which option 1 gets by
  convention. Fails KISS and the proposal's "keep existing JSON fields".

## Recommendation

Option 1. Focus is read where its other readers read it (the CLI, over `bridle-daemon`'s config
and focus modules), so it works without a daemon and with no new endpoint. Consistency comes
from the same bare form, the same `--schedule` and `--json` flags, the same `ScheduleSpan`, and
the same labels (`override`, `next change`, `ends`) in text. Show the existing override file
(including a pending one) since the proposal missed it. Trade-off accepted: two types, not one.

Also in the same change: update `docs/design/cli.md` (the `focus` line and the `budget` bullet),
`usage-and-budget.md` ("Seeing what applies") and roles-and-config.md ("Focus hours") to name the
bare form, and correct the "no CLI command" note on the override to say the file is written by
hand but read with `bridle focus`.

Verb naming: nothing to rename. Budget's change verbs stay, and focus still has no change verb
(its override is a hand-written file; a verb for it is out of scope, per the proposal).

## Rejected

- `show` verb on both: proposal rules it out (bare form exists on budget; one name per action).
- Option 2 (daemon endpoint): needs the daemon for a local read and ties machine config to a
  per-project daemon.
- Option 3 (one shared type): changes budget's JSON for little gain.
- Adding focus to `bridle status`: out of scope in the proposal; `status` already has the
  override line.
