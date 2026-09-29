## Purpose

What a profiling run produces, as plain data: the profile that records what was
observed of each column, and the proposals derived from it, each carrying the
evidence behind it and whether a person must decide on it before it may enter a
contract. It decides nothing and evaluates nothing — it is the durable record of
what the data actually showed, kept separate from what anyone concluded from it.

## Requirements

### Requirement: A profile reports the columns it observed, in observed order

A profile SHALL report every column found in the data, in the order the data
presents them, including any column whose observed type has no logical
equivalent. A column SHALL NOT be omitted from the profile on any ground.

#### Scenario: Every observed column appears in the profile

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data whose columns are order_id, quantity, unit_price, customer_note and status
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile names those columns in that order

### Requirement: A profile records each column's observed type and the logical type it maps to

For every column, a profile SHALL record the framework's own name for the
observed type as a readable label, and the library's logical type it maps to.
Where the observed type has no logical equivalent, the profile SHALL record the
label and SHALL report that no logical type was found, rather than guessing one
or omitting the column.

#### Scenario: An observed type maps to a logical type

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which a column holds \<observed\> values
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile reports that column's logical type as \<logical\>
- **AND** the profile records a readable label for its observed type

*Examples*:

| observed       | logical |
| -------------- | ------- |
| whole number   | integer |
| floating-point | float   |
| text           | string  |

#### Scenario: An observed type with no logical equivalent

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which a column holds values of a type the library has no logical type for
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile names that column
- **AND** the profile reports that no logical type was found for it
- **AND** the profile records a readable label for its observed type

### Requirement: A profile records the row count and each column's null and distinct counts

A profile SHALL record how many rows were observed, and for every column how
many of those rows were null and how many distinct values it held. These counts
SHALL be recorded for every column, including one whose observed type has no
logical equivalent.

#### Scenario: Counts are recorded for every column

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which exactly 3 rows hold a null "customer_note"
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile reports as many rows observed as the data holds
- **AND** the profile reports 3 nulls for "customer_note"
- **AND** every column in the profile reports a distinct count

### Requirement: A profile records observed extremes and bounded distinct values

For a column of logical type integer or float, a profile SHALL record the
smallest and largest non-null values observed. For a column of logical type
string, a profile SHALL record the distinct non-null values observed, collected
up to a fixed cap. Where a string column holds more distinct values than the
cap, the profile SHALL record that the cap was exceeded and SHALL NOT record a
value list for that column, so that a profile's size is bounded by construction
rather than by the data it was pointed at.

#### Scenario: Extremes are recorded for a numeric column

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which quantity holds values from 1 to 500
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile reports the smallest observed quantity as 1
- **AND** the profile reports the largest observed quantity as 500

#### Scenario: Distinct values are recorded for a string column within the cap

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which status holds exactly the five values pending, paid, shipped, delivered and cancelled
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile records exactly those five distinct values for status

#### Scenario: A string column beyond the cap records no value list

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data in which customer_note holds more distinct values than the cap
- **WHEN** the caller asks that engine to profile the data
- **THEN** the profile reports that customer_note exceeded the distinct-value cap
- **AND** the profile records no distinct-value list for customer_note

### Requirement: A profile records observations and derives nothing

A profile SHALL contain only what was observed of the data. It SHALL NOT carry a
proposed contract element, a decision, or a judgement about whether the data is
acceptable. Deriving proposals from a profile is a separate step, so that the
record of what the data showed survives independently of what anyone concluded
from it.

#### Scenario: A profile carries no proposal and no verdict

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the profile's fields
- **THEN** every field is an observation of the data — a count, an extreme, an observed value, or a type label
- **AND** no field holds a proposed contract element, an accept or reject decision, or a pass or fail verdict

### Requirement: A proposal names its element, its evidence, and whether it requires review

Every proposal SHALL name the contract element it proposes, SHALL carry the
evidence it was derived from — at minimum the number of rows observed — and
SHALL state whether it may enter a contract as observed or requires a person's
decision first.

#### Scenario: A proposal carries its element, evidence and classification

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data satisfying every declared type, nullability and constraint
- **WHEN** the caller derives proposals from a profile of that data
- **THEN** every proposal names the contract element it proposes
- **AND** every proposal reports as many rows observed as the data holds as its evidence
- **AND** every proposal states whether it requires review

### Requirement: Profiles and proposals are usable with no framework installed

Neither a profile nor a proposal SHALL require any dataframe or compute
framework to be importable. Only obtaining an engine to produce them requires
one.

#### Scenario: Profiles and proposals are usable with no framework installed

*Verification*: **executable**

- **GIVEN** no engine framework is importable
- **WHEN** a profile and a proposal are constructed and inspected
- **THEN** no exception is raised
