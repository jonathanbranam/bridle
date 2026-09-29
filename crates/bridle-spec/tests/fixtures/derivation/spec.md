## Purpose

Deriving a contract from data: what is proposed from a profile, which proposals
a person must decide before they may enter a contract, what is derived when the
data offers no evidence for a claim, and that accepting proposals yields an
ordinary contract the existing verification pass runs unchanged. Derivation
observes and proposes; it never writes a contract on its own authority.

## Requirements

### Requirement: Profiling modifies neither the data nor anything else

Profiling SHALL read the data and SHALL NOT alter it, reorder it, or write to
its source. Profiling the same data twice SHALL produce the same profile.

#### Scenario: Profiling leaves the data unchanged

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the caller asks that engine to profile the data twice
- **THEN** the data is unchanged
- **AND** both profiles report the same columns, counts, extremes and distinct values

### Requirement: A proposal is observed or requires review, decided by what the data can support

A proposal for a column's name or its logical type SHALL be classified as
observed, because both are read from the data's own schema. A proposal for
nullability, a range, or an allowed-values set SHALL be classified as requiring
review, because each generalises from whichever rows happened to be present and
may not describe the source as a whole. Nullability is in the second group for
the same reason a range is: an absence of nulls in the rows observed is not a
guarantee that the source forbids them.

#### Scenario: Structural proposals are observed and value proposals require review

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** every proposal for a column name or logical type is classified as observed
- **AND** every proposal for nullability, a range or an allowed-values set is classified as requiring review

### Requirement: A numeric column proposes a range at the observed extremes

For a column of logical type integer or float, derivation SHALL propose a range
whose lower bound is the smallest observed value and whose upper bound is the
largest, with both bounds inclusive. Inclusivity is not a choice here: the
observed extremes are values that occurred in the data, so a bound that excluded
them would contradict the evidence it was derived from.

#### Scenario: The proposed range spans the observed values inclusively

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which quantity holds values from 1 to 500
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** the proposed range for quantity has an inclusive lower bound of 1
- **AND** the proposed range for quantity has an inclusive upper bound of 500

### Requirement: A string column proposes an allowed-values set only within the distinct cap

For a column of logical type string whose distinct non-null values were fully
observed within the cap, derivation SHALL propose an allowed-values set holding
exactly those values. For a string column that exceeded the cap, derivation
SHALL propose no constraint, because the values it did see are a sample rather
than the set.

#### Scenario: A categorical column proposes its observed values

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which status holds exactly the five values pending, paid, shipped, delivered and cancelled
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** the proposed allowed values for status are exactly those five values

#### Scenario: A column beyond the cap proposes no constraint

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which customer_note holds more distinct values than the cap
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** no constraint is proposed for customer_note

### Requirement: Nullability is proposed from the nulls observed

A column in which at least one null was observed SHALL propose that nulls are
permitted. A column in which no null was observed SHALL propose that nulls are
not permitted, classified as requiring review like every other nullability
proposal.

#### Scenario: Nullability follows the nulls observed

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** the proposal for "customer_note" permits nulls
- **AND** the proposal for "order_id" does not permit nulls
- **AND** both proposals are classified as requiring review

### Requirement: Where the data offers no evidence, the weakest claim is derived

Where a column offers no evidence for a claim, derivation SHALL propose the
weakest claim the absent evidence cannot contradict: nulls permitted, no range,
and no allowed-values set. This SHALL hold wherever the evidence is missing —
whether because the data had no rows at all or because a column held nothing but
nulls — so that derivation never manufactures a claim the data did not make.

#### Scenario: A column holding only nulls derives no value claims

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data with rows in which every customer_note is null
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** the proposal for "customer_note" permits nulls
- **AND** no constraint is proposed for customer_note

### Requirement: Profiling data with no rows is reported and still derives a contract

Profiling data that has a schema but no rows SHALL NOT fail. The profile SHALL
report that no rows were observed, and derivation SHALL propose each column's
name and logical type from the schema and nothing further. Accepting those
proposals SHALL yield a valid contract, bearing types and permitting nulls,
carrying no range and no allowed-values set.

#### Scenario: An empty frame is flagged and yields a types-only contract

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data carrying the orders columns and no rows
- **WHEN** the caller profiles that data and accepts every proposal derived from it
- **THEN** the profile reports 0 rows observed
- **AND** the resulting contract declares every column of the data with its logical type
- **AND** every column of the resulting contract permits nulls
- **AND** no column of the resulting contract carries a constraint

### Requirement: A column with no logical type is reported and proposed for no contract element

Where a column's observed type has no logical equivalent, derivation SHALL
propose no contract element for it, because the library cannot yet describe it.
The profile SHALL still name the column and report why nothing was proposed, so
the gap is visible rather than silently absent from the resulting contract.

#### Scenario: An undescribable column is named but yields no contract element

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which a column holds values of a type the library has no logical type for
- **WHEN** the caller profiles that data and accepts every proposal derived from it
- **THEN** the profile names that column
- **AND** the profile reports that no logical type was found for it
- **AND** the resulting contract declares no column of that name

### Requirement: A proposal requiring review is individually acceptable and rejectable

Proposals requiring review SHALL be acceptable or rejectable one at a time, not
only as a batch, so that a person may keep a derived range and decline a derived
categorical set in the same review.

#### Scenario: One proposal is rejected and the rest are kept

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the caller derives proposals from a profile of that data
- **AND** the caller rejects the proposed range for quantity and accepts every other proposal
- **THEN** the resulting contract declares no range for quantity
- **AND** the resulting contract declares the proposed allowed values for status

### Requirement: Rejecting a proposal falls back to the weakest claim

Rejecting a proposal SHALL leave the contract making no claim in its place,
rather than leaving the element unset or the contract unbuildable. A rejected
range or allowed-values proposal SHALL yield a column carrying no constraint,
and a rejected proposal that nulls are forbidden SHALL yield a column permitting
them — the same weakest claim derivation reaches for when the data offers no
evidence.

#### Scenario: Rejecting a non-nullable proposal permits nulls

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which exactly 0 rows hold a null "order_id"
- **WHEN** the caller derives proposals from a profile of that data
- **AND** the caller rejects the proposal that "order_id" forbids nulls
- **THEN** the resulting contract declares that "order_id" permits nulls

### Requirement: A contract derived from data verifies clean against that data

Accepting every proposal derived from data whose columns all carry a logical
type SHALL yield a contract that the existing verification pass runs unchanged,
and verifying it against the data it was derived from SHALL report that the data
satisfies the contract. A derivation that produced a contract its own source
data violates would be reporting something untrue about what it observed.

#### Scenario: The derived contract accepts the data it came from

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the caller profiles that data and accepts every proposal derived from it
- **AND** the resulting contract is verified against that data
- **THEN** every check in the report has outcome passed
- **AND** the report's verdict is that the data satisfies the contract

### Requirement: Derivation proposes, and never writes a contract on its own authority

There SHALL be no path from data to a contract that does not pass through
proposals the caller acts on. Derivation SHALL NOT alter a contract the caller
already holds, and SHALL NOT persist, install, or register the contract it
helped produce anywhere.

#### Scenario: No path turns data into a contract without a decision

*Verification*: **non-executable**

- **WHEN** a reviewer inspects every public path from data to a contract
- **THEN** each one returns proposals the caller must accept or reject before a contract exists
- **AND** none writes to, replaces, or amends a contract the caller already holds
- **AND** none persists the resulting contract to any location
