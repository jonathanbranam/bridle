# Calc

## Purpose

A tiny calculator.

### Requirement: Adding works                                  {#r-aa01}

The calculator SHALL add.

#### Scenario: Add two numbers                                 {#s-aa01}
*Verification*: **executable** @fast

- **GIVEN** a calculator
- **WHEN** it adds 2 and 3
- **THEN** the result is 5

#### Scenario: Add is fast                                     {#s-aa02}
*Verification*: **non-executable**

Measured by hand.

### Requirement: Multiplying works                             {#r-aa02}

The calculator SHALL multiply.

#### Scenario: Multiply pairs                                   {#s-aa03}
*Verification*: **executable** @slow

- **GIVEN** a calculator
- **WHEN** it multiplies \<a\> and \<b\>
- **THEN** the result is \<product\>

*Examples*:

| a | b | product |
| - | - | ------- |
| 2 | 3 | 6       |
| 4 | 5 | 20      |
