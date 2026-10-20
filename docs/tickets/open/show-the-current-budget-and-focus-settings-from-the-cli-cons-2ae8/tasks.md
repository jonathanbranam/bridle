# Tasks

## 1. Specs

- [ ] 1.1 Merge specs.md into design/specs, then run `bridle spec id` and `bridle spec check` (both pass)

## 2. Focus status type and builder

Touches `crates/bridle-api/src/types.rs`, `crates/bridle/src/focus.rs`.

- [ ] 2.1 Add `FocusStatus` (current period, next change, override, schedule of name/span/mode) to `bridle-api/src/types.rs`, reusing `ScheduleSpan`, with UTC times (check: serde round-trip unit test, and the JSON keys are `next_change`, `override`, `schedule`, with `span` of `days`, `start`, `end`)
- [ ] 2.2 Add a pure builder in `crates/bridle/src/focus.rs` that takes `home` and `now` and fills a `FocusStatus` from `focus_periods`, `focus_end` and `read_override` (check: unit tests for inside a period at 15:30 on a Tuesday, outside every period on a Saturday noon, and no periods configured)
- [ ] 2.3 Make the builder report an active override and a pending (delayed-start) override with start, end and reason, and none when absent (check: unit tests for the three cases)

## 3. `bridle focus` read form

Touches `crates/bridle/src/cli.rs`, `crates/bridle/src/focus.rs`, `crates/bridle/src/commands/usage.rs` (labels only).

- [ ] 3.1 Make `FocusArgs.action` optional and add `--json`, so bare `bridle focus` runs the read and the `gate` and `reply` hooks parse as before (check: CLI parse tests for bare, `--json`, `gate`, `reply`)
- [ ] 3.2 Print the text form: period in effect with what it does and when it ends, next change, override lines, a "focus hours are off" line with no periods, and nothing about an override when none, all in budget's labels (`override`, `next change`, `ends`) and local time (check: integration tests for the inside, outside, no-periods, active, pending and no-override scenarios)
- [ ] 3.3 Add `--schedule`, listing every period with name, days, hours and mode (check: integration test with a weekday work period and an evening period lists both)
- [ ] 3.4 Print `FocusStatus` as one JSON object for `--json` (check: integration test with periods and an override parses and has the shared keys)
- [ ] 3.5 Check the read works with no daemon running (check: integration test with no daemon exits 0 and shows the settings)

## 4. Budget alignment

Touches `crates/bridle/src/commands/usage.rs`, `crates/bridle-api/src/types.rs` (only if a key must be added).

- [ ] 4.1 Make budget's text labels and JSON names for the shared parts (`override`, `next change`/`next_change`, `schedule` with `span`) match focus's, removing no existing key (check: test that `bridle budget --json` keeps every pre-change key and has the shared ones; test that both overrides use the same words)
- [ ] 4.2 Confirm `bridle budget` and `bridle budget --schedule` still give what they gave (check: existing budget tests pass unchanged)
- [ ] 4.3 Confirm `focus gate` is silent outside every focus period and succeeds (check: test for the hook with the clock outside all periods prints nothing)

## 5. Docs

Touches `docs/design/cli.md`, `docs/design/agent-host/usage-and-budget.md`, `docs/design/agent-host/roles-and-config.md`.

- [ ] 5.1 Update the `focus` line and `budget` bullet in `docs/design/cli.md` to name the bare form, `--schedule` and `--json` (check: `grep` shows both commands described; `just check` passes)
- [ ] 5.2 Update "Seeing what applies" in usage-and-budget.md and "Focus hours" in roles-and-config.md, and correct the override note to say the file is written by hand and read with `bridle focus` (check: the "no CLI command" wording is gone; `just check` passes)
