# Widgets

## Requirements

### Requirement: Widgets are counted {#r-7fa2}

The system SHALL count widgets.

#### Scenario: Counting two widgets {#s-b310}

*Verification*: **executable** @smoke

- **GIVEN** two widgets
- **WHEN** they are counted
- **THEN** the count is 2

#### Scenario: Counting reads well {#s-b311}

*Verification*: **non-executable**

A human reads the output and agrees.

### Requirement: Widgets have colours {#r-7fa4}

The system SHALL colour widgets.

#### Scenario: Colouring {#s-b312}

*Verification*: **executable**

- **WHEN** a widget is coloured \<colour\>
- **THEN** it reports \<colour\>

*Examples*:

| colour |
| ------ |
| red    |
| blue   |
