## design/specs/period-status.md

### Requirement: Focus settings can be read from the CLI

The system SHALL let the user read their focus settings with `bridle focus` and no subcommand: the configured periods, the period in effect now and when it ends, and when the next change comes. It SHALL work without a running daemon.

#### Scenario: Inside a focus period
*Verification*: **executable**
- **GIVEN** a focus period that runs Monday to Friday, 8:00 to 18:00, and the clock is at 15:30 on a Tuesday
- **WHEN** the user runs `bridle focus`
- **THEN** the output tells them that period is in effect, what it does, when it ends, and when focus next changes

#### Scenario: Outside every period
*Verification*: **executable**
- **GIVEN** focus periods that run on weekdays only, and the clock is at noon on a Saturday
- **WHEN** the user runs `bridle focus`
- **THEN** the output tells them no period is in effect, and when focus next changes

#### Scenario: No focus periods configured
*Verification*: **executable**
- **GIVEN** a config with no focus periods
- **WHEN** the user runs `bridle focus`
- **THEN** the output tells them focus hours are off, and the command succeeds

#### Scenario: Daemon is down
*Verification*: **executable**
- **GIVEN** focus periods are configured and no daemon is running
- **WHEN** the user runs `bridle focus`
- **THEN** the settings are still shown, and the command succeeds

### Requirement: Focus shows its override

The system SHALL show an active or pending focus override in the output of `bridle focus`, with its start, its end and the reason it was given, and SHALL show nothing about an override when there is none.

#### Scenario: Active override
*Verification*: **executable**
- **GIVEN** an override that is in force now and ends at 17:00, given the reason "deploy is broken"
- **WHEN** the user runs `bridle focus`
- **THEN** the output tells them an override is active until 17:00 and why

#### Scenario: Pending override
*Verification*: **executable**
- **GIVEN** an override whose delayed start has not yet arrived
- **WHEN** the user runs `bridle focus`
- **THEN** the output tells them an override is pending, and when it starts and ends

#### Scenario: No override
*Verification*: **executable**
- **GIVEN** no focus override has been set
- **WHEN** the user runs `bridle focus`
- **THEN** the output does not mention an override

### Requirement: Focus lists every period with --schedule

The system SHALL list every configured focus period, with its name, the days and hours it covers and what it does, when the user runs `bridle focus --schedule`.

#### Scenario: Two periods
*Verification*: **executable**
- **GIVEN** a weekday work period and a separate evening period are configured
- **WHEN** the user runs `bridle focus --schedule`
- **THEN** both periods are listed, each with its name, days, hours and what it does

### Requirement: Budget and focus read the same way

The system SHALL show the same facts about budget as about focus when the user runs `bridle budget` with no verb: the periods configured (with `--schedule`), the period in effect now and when it ends, when the next change comes, and any hold or override. The two commands SHALL name the shared facts in the same words, so a user who learns one can read the other.

#### Scenario: Both overrides are visible alike
*Verification*: **executable**
- **GIVEN** a budget override and a focus override are both active
- **WHEN** the user runs `bridle budget` and then `bridle focus`
- **THEN** each output shows its override and the upcoming change in the same words

#### Scenario: Existing budget invocations keep working
*Verification*: **executable**
- **GIVEN** a running daemon
- **WHEN** the user runs `bridle budget` and `bridle budget --schedule`
- **THEN** both succeed and still give everything they gave before this change

### Requirement: Shared JSON key names

The system SHALL print machine-readable settings with `--json` on both `bridle focus` and `bridle budget`, and the parts they share SHALL use the same key names: `next_change`, `override`, and a `schedule` list whose items have a `span` of `days`, `start` and `end`. Times in JSON SHALL be UTC. These names are the contract scripts rely on and SHALL NOT change without intent. Every key budget's JSON had before this change SHALL stay, with the same meaning.

#### Scenario: Focus JSON uses the shared names
*Verification*: **executable**
- **GIVEN** focus periods and an override are configured
- **WHEN** the user runs `bridle focus --json`
- **THEN** one JSON object is printed whose period list, next change and override use the shared key names

#### Scenario: Budget JSON keeps its old keys and adopts the shared names
*Verification*: **executable**
- **GIVEN** a daemon with a budget schedule and a hold
- **WHEN** the user runs `bridle budget --json`
- **THEN** every key it had before is still there with the same meaning, and the shared parts use the same key names as the focus JSON

### Requirement: Focus hooks are unchanged

The system SHALL keep the `focus gate` and `focus reply` hooks behaving as before, including staying silent when they have nothing to say, and neither SHALL print the settings.

#### Scenario: Hook stays silent outside focus
*Verification*: **executable**
- **GIVEN** the clock is outside every focus period
- **WHEN** the `focus gate` hook runs
- **THEN** it prints nothing and the command succeeds
