## Purpose

Evaluating a table contract against data and reporting what was found: which
checks ran, what each one observed, how severe a violation is, and whether the
data satisfies the contract overall. Verification observes and reports; it never
modifies the data or the contract.

## Requirements

### Requirement: Verification evaluates every check and returns a report

Verification SHALL evaluate all applicable checks and return a structured
report. It SHALL NOT stop at the first violation, and SHALL NOT raise on
violating data — a violation is a reported outcome, not an exception. Whether a
failing verdict stops anything is the caller's decision, not the library's.

#### Scenario: Violations in several columns are all reported

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data violating the range on "quantity" and the allowed values on "status"
- **WHEN** the contract is verified against that data
- **THEN** no exception is raised
- **AND** the report contains a failed check for "quantity" with rule range
- **AND** the report contains a failed check for "status" with rule allowed-values

#### Scenario: Clean data produces a passing report

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the contract is verified against that data
- **THEN** every check in the report has outcome passed
- **AND** the report's verdict is that the data satisfies the contract

### Requirement: Verification modifies neither the data nor the contract

Verification SHALL leave the caller's data and the contract unchanged.

#### Scenario: Neither input is modified

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data violating the range on "quantity"
- **WHEN** the contract is verified against that data
- **THEN** the data is unchanged
- **AND** the contract is unchanged

### Requirement: Each check result identifies its column, rule, and outcome

Every check in the report SHALL identify the column it concerns, which rule was
evaluated, and an outcome of passed, failed, or skipped. A skipped check SHALL
carry the reason it could not be evaluated. A check that could not run SHALL be
reported as skipped — never omitted from the report, and never reported as
passing.

#### Scenario: A check result is self-describing

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the contract is verified against that data
- **THEN** every check in the report names a column and a rule
- **AND** every check in the report has outcome passed, failed or skipped

#### Scenario: A skipped check carries a reason

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data carrying every declared column except "status"
- **WHEN** the contract is verified against that data
- **THEN** the type check for "status" has outcome skipped
- **AND** that check carries a reason naming "status"

### Requirement: The report's checks are a function of the contract, not of the data

The set and order of checks in a report SHALL be determined by the contract
alone. Two runs against different data SHALL produce reports carrying the same
checks in the same order, differing only in outcome.

#### Scenario: Different data produces the same checks

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data satisfying every declared type, nullability and constraint
- **AND** data violating the range on "quantity"
- **WHEN** the contract is verified against each of them
- **THEN** both reports hold the same checks in the same order
- **AND** the two reports differ only in outcomes, violating row counts and samples

### Requirement: A failed check reports the violating row count and a bounded sample

A failed check SHALL report how many rows violated it, and SHALL include a
sample of the offending values bounded to at most five. The sample SHALL be
deterministic for the same data, so that repeated runs produce identical
reports.

#### Scenario: Violating rows are counted

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which exactly 3 rows hold a "quantity" of 0
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "quantity" with rule range reports 3 violating rows

#### Scenario: The sample is bounded to five values

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which exactly 9 rows hold a "status" outside the allowed values
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "status" with rule allowed-values reports 9 violating rows
- **AND** that check's sample holds at most 5 offending values

#### Scenario: Repeated verification produces an identical report

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data violating the range on "quantity" and the allowed values on "status"
- **WHEN** the contract is verified against that data twice
- **THEN** both reports hold the same checks with the same outcomes, violating row counts and samples

### Requirement: Every check carries a severity and the verdict derives from errors alone

Every check SHALL carry a severity of error or warning. The report's overall
verdict SHALL be derived from failed checks of error severity only: failing
warnings SHALL NOT make the verdict fail.

#### Scenario: Warnings alone do not fail the verdict

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data satisfying every declared type, nullability and constraint
- **AND** that data carries one extra column named "internal_id"
- **WHEN** the contract is verified against that data
- **THEN** the report contains a failed check with severity warning naming "internal_id"
- **AND** the report contains no failed check with severity error
- **AND** the report's verdict is that the data satisfies the contract

#### Scenario: An error fails the verdict

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data violating the range on "quantity"
- **WHEN** the contract is verified against that data
- **THEN** the report contains a failed check with severity error
- **AND** the report's verdict is that the data does not satisfy the contract

### Requirement: A column present in the data but absent from the contract is a warning

Data carrying a column the contract does not declare SHALL be reported as a
failed check of warning severity, naming the unexpected column. It SHALL NOT
fail the verdict, and SHALL NOT be passed over silently.

#### Scenario: An unexpected column is reported as a warning

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data satisfying every declared type, nullability and constraint
- **AND** that data carries one extra column named "internal_id"
- **WHEN** the contract is verified against that data
- **THEN** the report contains a failed check with severity warning and rule unexpected-column naming "internal_id"

### Requirement: A column declared by the contract but absent from the data is an error

A declared column missing from the data SHALL be reported as a failed check of
error severity. Every other check concerning that column SHALL be reported as
skipped.

#### Scenario: A missing column fails and skips its dependents

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data carrying every declared column except "status"
- **WHEN** the contract is verified against that data
- **THEN** the report contains a failed check with severity error and rule column-present naming "status"
- **AND** the type, nullability and allowed-values checks for "status" have outcome skipped
- **AND** the report's verdict is that the data does not satisfy the contract

### Requirement: A column whose type does not match its declared logical type is an error

Verification SHALL compare each present column's type against its declared
logical type and report a mismatch as a failed check of error severity. Any
width of an engine's integer type SHALL satisfy the integer logical type, and
any width of its floating-point type SHALL satisfy the float logical type.
Verification SHALL NOT coerce: integer data does not satisfy a float column, and
a mismatched type SHALL cause the constraint checks for that column to be
reported as skipped.

#### Scenario: A matching type of a different width passes

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which "order_id" holds integers of a different width from the engine's default
- **WHEN** the contract is verified against that data
- **THEN** the check for "order_id" with rule type has outcome passed

#### Scenario: A mismatched type is an error and skips that column's constraints

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which "unit_price" holds integer values rather than floating-point values
- **WHEN** the contract is verified against that data
- **THEN** the check for "unit_price" with rule type has outcome failed with severity error
- **AND** the range check for "unit_price" has outcome skipped

### Requirement: Nulls are governed by nullability alone

A null in a column declared as not permitting nulls SHALL be reported as a
failed check of error severity, with the count of null rows. Nulls in a column
that permits them SHALL pass, and in both cases nulls SHALL be excluded from
range and allowed-values evaluation rather than counted as constraint
violations.

#### Scenario: Nulls in a non-nullable column are an error

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which exactly 2 rows hold a null "status"
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "status" with rule nullability reports 2 violating rows
- **AND** that check has severity error

#### Scenario: Nulls in a nullable column do not violate its constraints

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which "customer_note" holds nulls alongside ordinary text
- **WHEN** the contract is verified against that data
- **THEN** the check for "customer_note" with rule nullability has outcome passed
- **AND** no check for "customer_note" counts a null as a violation

#### Scenario: Nulls are excluded from constraint evaluation

*Verification*: **executable** @engine

- **GIVEN** the orders contract with "status" permitting nulls
- **AND** data in which "status" holds nulls alongside values drawn from the allowed set
- **WHEN** the contract is verified against that data
- **THEN** the check for "status" with rule allowed-values has outcome passed

### Requirement: Values outside a declared range are an error

For a column carrying a range constraint, verification SHALL report values
outside the declared bounds as a failed check of error severity.

#### Scenario: Out-of-range values are counted and sampled

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which exactly 2 rows hold a "quantity" above 999
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "quantity" with rule range reports 2 violating rows
- **AND** that check's sample holds the offending values

#### Scenario: An open-ended range constrains only the end it declares

*Verification*: **executable** @engine

- **GIVEN** the orders contract in which "unit_price" declares a lower bound and no upper bound
- **AND** data in which "unit_price" holds arbitrarily large values
- **WHEN** the contract is verified against that data
- **THEN** the check for "unit_price" with rule range has outcome passed

### Requirement: A range bound is evaluated according to its declared inclusivity

Verification SHALL evaluate each range bound according to the inclusivity that
bound declares. A value exactly equal to an inclusive bound SHALL satisfy the
range; a value exactly equal to an exclusive bound SHALL be a violation. The two
bounds SHALL be evaluated independently, so a range closed at one end and open
at the other SHALL admit its lower boundary value and reject its upper one, or
the reverse.

#### Scenario: A value on an inclusive bound satisfies the range

*Verification*: **executable** @engine

- **GIVEN** the orders contract in which "quantity" declares an inclusive lower bound of 1 and an inclusive upper bound of 999
- **AND** data in which every "quantity" is \<value\>
- **WHEN** the contract is verified against that data
- **THEN** the check for "quantity" with rule range has outcome passed

*Examples*:

| value |
| ----- |
| 1     |
| 999   |

#### Scenario: A value on an exclusive bound violates the range

*Verification*: **executable** @engine

- **GIVEN** the orders contract in which "unit_price" declares an exclusive lower bound of 0.0
- **AND** data in which exactly 2 rows hold a "unit_price" of 0.0
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "unit_price" with rule range reports 2 violating rows

#### Scenario: A half-open range admits one boundary and rejects the other

*Verification*: **executable** @engine

- **GIVEN** a contract whose "discount" column declares an inclusive lower bound of 0.0 and an exclusive upper bound of 1.0
- **AND** data in which "discount" holds both 0.0 and 1.0
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "discount" with rule range reports 1 violating rows
- **AND** that check's sample holds the offending values

### Requirement: Values outside a declared allowed-values set are an error

For a column carrying an allowed-values constraint, verification SHALL report
any value not in the declared set as a failed check of error severity.

#### Scenario: Disallowed values are counted and sampled

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data in which exactly 2 rows hold a "status" outside the allowed values
- **WHEN** the contract is verified against that data
- **THEN** the failed check for "status" with rule allowed-values reports 2 violating rows
- **AND** that check's sample holds the offending values

#### Scenario: A column with no constraint reports no constraint check

*Verification*: **executable** @engine

- **GIVEN** the orders contract
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the contract is verified against that data
- **THEN** the report contains no constraint check for "customer_note"
- **AND** the report contains a type check and a nullability check for "customer_note"
