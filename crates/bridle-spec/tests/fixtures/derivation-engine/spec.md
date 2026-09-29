## Purpose

The profiling side of the engine boundary: what an engine must answer for data
to be profiled, that those answers cross back as plain values rather than
framework objects, and that an engine which has not implemented profiling may
say so rather than being unusable for anything. Framework isolation and
per-framework optional installs are already required of every engine and are not
restated here.

## Requirements

### Requirement: An engine profiles data it understands

An engine SHALL accept data of a type it supports and SHALL return an
engine-agnostic profile of it, requiring no contract as input. Data of a type
the engine does not support SHALL be rejected with an error naming the types it
accepts, on the same terms as verification.

#### Scenario: Profiling through an engine

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **AND** data of a type that engine understands
- **WHEN** the caller asks that engine to profile the data
- **THEN** a profile is returned

#### Scenario: An engine rejects data it does not understand

*Verification*: **executable** @engine @polars

- **GIVEN** an engine constructed by the caller
- **WHEN** the caller asks that engine to profile an object of an unsupported type
- **THEN** it fails with an error naming the data types that engine accepts

### Requirement: An engine that has not implemented a capability declines it

An engine SHALL remain constructible and usable for the capabilities it does
implement even when it implements others not at all. Asking an engine for a
capability it has not implemented SHALL fail with an error owned by this
library, so that a caller catching the library's own error hierarchy catches it
along with everything else. Implementing verification SHALL NOT oblige an engine
to implement profiling, so that a capability may land one engine at a time.

#### Scenario: An engine implementing only verification is constructible

*Verification*: **executable**

- **GIVEN** an engine that implements only the observations verification needs
- **WHEN** that engine is constructed
- **THEN** no exception is raised

#### Scenario: Asking for an unimplemented capability fails with a library error

*Verification*: **executable**

- **GIVEN** an engine that implements only the observations verification needs
- **WHEN** the caller asks that engine to profile data
- **THEN** it fails with an error belonging to this library's error hierarchy
- **AND** the error names the capability that engine does not implement

#### Scenario: Declining one capability leaves the others working

*Verification*: **executable**

- **GIVEN** an engine that implements only the observations verification needs
- **AND** the orders contract
- **WHEN** the caller asks that engine to verify the contract against data it understands
- **THEN** a report is returned

### Requirement: The caller names the engine that profiles, and the library never infers one

Profiling SHALL be reached through an engine the caller constructs and names, on
the same terms as verification. The library SHALL NOT infer an engine from the
type of the data it is handed, and SHALL NOT provide a registry or discovery
mechanism that selects one.

#### Scenario: No dispatch path selects an engine for profiling

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the library's public surface
- **THEN** every path to profiling requires the caller to name an engine
- **AND** no function inspects the data's type to choose one

### Requirement: Profiling observations cross the boundary as plain values

Every observation an engine answers for profiling SHALL return plain Python
values — counts, scalars, logical types, readable labels, and tuples of those.
No dataframe, series, dtype object, or other framework type SHALL appear in what
crosses back, so that everything downstream of the boundary stays framework-free
and identical for every engine.

#### Scenario: No framework type appears in a profiling observation

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the signatures and return values of the observations profiling requires of an engine
- **THEN** each returns only counts, scalars, logical types, readable labels, or tuples of those
- **AND** none names or returns a type belonging to a dataframe or compute framework

### Requirement: The Polars engine profiles both eager and lazy in-memory frames

The Polars engine SHALL profile an in-memory frame whether it is eager or lazy,
and SHALL produce the same profile for data that differs only in which of the
two it is.

#### Scenario: Eager and lazy frames profile identically

*Verification*: **executable** @polars

- **GIVEN** an eager frame and a lazy frame holding the same data
- **WHEN** the Polars engine profiles each of them
- **THEN** both profiles report the same columns, counts, extremes and distinct values
