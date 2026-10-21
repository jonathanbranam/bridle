# Period status

How `bridle focus` and `bridle budget` show the periods they apply, the one in effect, the next
change and any override. The code is `crates/bridle/src/focus.rs` (focus) and
`crates/bridle/src/commands/usage.rs` (budget); the design is in
`docs/tickets/open/show-the-current-budget-and-focus-settings-from-the-cli-cons-2ae8/design.md`.

## Requirements

### Requirement: Focus settings can be read from the CLI  {#r-c478}

The system SHALL let the user read their focus settings with `bridle focus` and no subcommand: the configured periods, the period in effect now and when it ends, and when the next change comes. It SHALL work without a running daemon.

#### Scenario: Inside a focus period  {#s-2596}
*Verification*: **executable**
- **GIVEN** a focus period that runs Monday to Friday, 8:00 to 18:00
- **AND** the clock is at 15:30 on a Tuesday
- **WHEN** the user runs bridle focus
- **THEN** the output tells them that period is in effect
- **AND** the output tells them what the period does
- **AND** the output tells them when it ends
- **AND** the output tells them when focus next changes

#### Scenario: Outside every period  {#s-dea9}
*Verification*: **executable**
- **GIVEN** focus periods that run on weekdays only
- **AND** the clock is at noon on a Saturday
- **WHEN** the user runs bridle focus
- **THEN** the output tells them no period is in effect
- **AND** the output tells them when focus next changes

#### Scenario: No focus periods configured  {#s-fd51}
*Verification*: **executable**
- **GIVEN** a config with no focus periods
- **WHEN** the user runs bridle focus
- **THEN** the output tells them focus hours are off
- **AND** the command succeeds

#### Scenario: Daemon is down  {#s-2fe2}
*Verification*: **executable**
- **GIVEN** focus periods are configured
- **AND** no daemon is running
- **WHEN** the user runs bridle focus
- **THEN** the settings are still shown
- **AND** the command succeeds

### Requirement: Focus shows its override  {#r-7d36}

The system SHALL show an active or pending focus override in the output of `bridle focus`, with its start, its end and the reason it was given, and SHALL show nothing about an override when there is none.

#### Scenario: Active override  {#s-82ce}
*Verification*: **executable**
- **GIVEN** an override that is in force now
- **AND** it ends at 17:00
- **AND** it was given the reason "deploy is broken"
- **WHEN** the user runs bridle focus
- **THEN** the output tells them an override is active until 17:00
- **AND** the output tells them why

#### Scenario: Pending override  {#s-4231}
*Verification*: **executable**
- **GIVEN** an override whose delayed start has not yet arrived
- **WHEN** the user runs bridle focus
- **THEN** the output tells them an override is pending
- **AND** the output tells them when it starts and ends

#### Scenario: No override  {#s-6dc3}
*Verification*: **executable**
- **GIVEN** no focus override has been set
- **WHEN** the user runs bridle focus
- **THEN** the output does not mention an override

### Requirement: Focus lists every period with --schedule  {#r-0c73}

The system SHALL list every configured focus period, with its name, the days and hours it covers and what it does, when the user runs `bridle focus --schedule`.

#### Scenario: Two periods  {#s-8f8e}
*Verification*: **executable**
- **GIVEN** a weekday work period is configured
- **AND** a separate evening period is configured
- **WHEN** the user runs bridle focus --schedule
- **THEN** both periods are listed
- **AND** each shows its name, days, hours and what it does

### Requirement: Budget and focus read the same way  {#r-bb10}

The system SHALL show the same facts about budget as about focus when the user runs `bridle budget` with no verb: the periods configured (with `--schedule`), the period in effect now and when it ends, when the next change comes, and any hold or override. The two commands SHALL name the shared facts in the same words, so a user who learns one can read the other.

#### Scenario: Both overrides are visible alike  {#s-23ed}
*Verification*: **executable**
- **GIVEN** a budget override is active
- **AND** a focus override is active
- **WHEN** the user runs bridle budget
- **AND** the user runs bridle focus
- **THEN** each output shows its override
- **AND** each output shows the upcoming change
- **AND** the two use the same words for both

#### Scenario: Existing budget invocations keep working  {#s-5bf6}
*Verification*: **executable**
- **GIVEN** a running daemon
- **WHEN** the user runs bridle budget
- **AND** the user runs bridle budget --schedule
- **THEN** both succeed
- **AND** both still give everything they gave before this change

### Requirement: Shared JSON key names  {#r-e178}

The system SHALL print machine-readable settings with `--json` on both `bridle focus` and `bridle budget`, and the parts they share SHALL use the same key names: `next_change`, `override`, and a `schedule` list whose items have a `span` of `days`, `start` and `end`. Times in JSON SHALL be UTC. These names are the contract scripts rely on and SHALL NOT change without intent. Every key budget's JSON had before this change SHALL stay, with the same meaning.

#### Scenario: Focus JSON uses the shared names  {#s-e08c}
*Verification*: **executable**
- **GIVEN** focus periods are configured
- **AND** a focus override is set
- **WHEN** the user runs bridle focus --json
- **THEN** one JSON object is printed
- **AND** its period list, next change and override use the shared key names

#### Scenario: Budget JSON keeps its old keys and adopts the shared names  {#s-3a11}
*Verification*: **executable**
- **GIVEN** a daemon with a budget schedule
- **AND** a hold is active
- **WHEN** the user runs bridle budget --json
- **THEN** every key it had before is still there with the same meaning
- **AND** the shared parts use the same key names as the focus JSON

### Requirement: Focus hooks are unchanged  {#r-faed}

The system SHALL keep the `focus gate` and `focus reply` hooks behaving as before, including staying silent when they have nothing to say, and neither SHALL print the settings.

#### Scenario: Hook stays silent outside focus  {#s-ecc8}
*Verification*: **executable**
- **GIVEN** the clock is outside every focus period
- **WHEN** the focus gate hook runs
- **THEN** it prints nothing
- **AND** the command succeeds
