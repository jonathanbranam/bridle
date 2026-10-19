## design/specs/period-status.md

### Requirement: Focus settings can be read from the CLI

The system SHALL print the configured focus periods, the period in effect now with when it ends, and any focus override, when `bridle focus` is run with no subcommand, without needing a running daemon.

#### Scenario: Inside a focus period
*Verification*: **executable**
- **GIVEN** a `[[focus]]` period `work` (mode `quiet`, mon-fri 08:00-18:00) and a clock inside it
- **WHEN** the user runs `bridle focus`
- **THEN** the output has a `now` line naming the period `work` and its mode, an `ends` line with its end time in local time, and a `next change` line

#### Scenario: Outside every period
*Verification*: **executable**
- **GIVEN** `[[focus]]` periods and a clock outside all of them
- **WHEN** the user runs `bridle focus`
- **THEN** the output has the line `now      none` and a `next change` line

#### Scenario: No focus periods configured
*Verification*: **executable**
- **GIVEN** a config with no `[[focus]]` entries
- **WHEN** the user runs `bridle focus`
- **THEN** the output is `no [[focus]] periods; focus hours are off` and the exit code is 0

#### Scenario: Daemon is down
*Verification*: **executable**
- **GIVEN** `[[focus]]` periods and no daemon running
- **WHEN** the user runs `bridle focus`
- **THEN** the settings are printed and the exit code is 0

### Requirement: Focus shows its override

The system SHALL show an active or pending override from `~/.bridle/focus-override.toml` in the output of `bridle focus`, with its end time and reason, on a line labelled `override`.

#### Scenario: Active override
*Verification*: **executable**
- **GIVEN** an override file valid now, ending 17:00, with reason "deploy is broken"
- **WHEN** the user runs `bridle focus`
- **THEN** an `override` line shows it as active until that end time, with the reason

#### Scenario: Pending override
*Verification*: **executable**
- **GIVEN** an override file whose delayed start has not yet arrived
- **WHEN** the user runs `bridle focus`
- **THEN** the `override` line shows it as pending, with its start and end times

#### Scenario: No override file
*Verification*: **executable**
- **GIVEN** no override file
- **WHEN** the user runs `bridle focus`
- **THEN** no `override` line is printed

### Requirement: Focus lists every period with --schedule

The system SHALL print one line per configured focus period, with its name, days, time span and mode, when `bridle focus --schedule` is run.

#### Scenario: Two periods
*Verification*: **executable**
- **GIVEN** two `[[focus]]` periods
- **WHEN** the user runs `bridle focus --schedule`
- **THEN** two lines are printed, each with the period's name, days, start-end span and mode

### Requirement: Focus and budget settings read as JSON with shared names

The system SHALL print machine-readable settings with `--json` on both `bridle focus` and `bridle budget`, and the shared parts SHALL use the same key names: `next_change`, `override`, and `schedule` items with a `span` of `{days, start, end}`. Times in JSON SHALL be UTC.

#### Scenario: Focus JSON
*Verification*: **executable**
- **GIVEN** `[[focus]]` periods and an override
- **WHEN** the user runs `bridle focus --json`
- **THEN** the output is one JSON object with `current`, `next_change`, `override` and `schedule`, where each `schedule` item has `name`, `span` (`days`, `start`, `end`) and `mode`

#### Scenario: Budget JSON keeps its fields
*Verification*: **executable**
- **GIVEN** a daemon with a budget schedule and a hold
- **WHEN** the user runs `bridle budget --json`
- **THEN** every field the output had before this change is still present with the same meaning, and `next_change` and `override` use the same names as the focus output

### Requirement: Budget settings stay readable in the same shape

The system SHALL keep `bridle budget` with no verb printing the governor state, the period in effect and its end, any hold or override, and SHALL keep `bridle budget --schedule` printing every period, using the labels `override`, `next change` and `span` spelled as in `bridle focus`.

#### Scenario: Shared labels
*Verification*: **executable**
- **GIVEN** a budget override and a focus override both active
- **WHEN** the user runs `bridle budget` and `bridle focus`
- **THEN** both outputs label the override line `override` and the upcoming change `next change`

#### Scenario: Existing invocations still work
*Verification*: **executable**
- **GIVEN** a running daemon
- **WHEN** the user runs `bridle budget` and `bridle budget --schedule`
- **THEN** both succeed and print what they printed before this change, plus the shared labels

### Requirement: Focus hooks are unchanged

The system SHALL keep `bridle focus gate` and `bridle focus reply` behaving and staying silent as before, and neither SHALL print the settings read.

#### Scenario: Hook still silent outside focus
*Verification*: **executable**
- **GIVEN** a clock outside every focus period
- **WHEN** the `focus gate` hook runs
- **THEN** it prints nothing and exits 0
