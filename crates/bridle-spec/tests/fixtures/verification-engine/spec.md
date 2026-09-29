## Purpose

The boundary between the engine-agnostic contract and the framework that
actually holds the data. It keeps framework types out of the contract and the
report, keeps each supported framework an independent optional install, and
makes adding a further engine additive rather than a change to everything else.

## Requirements

### Requirement: An engine verifies a contract against data it understands

An engine SHALL accept an engine-agnostic contract and data of a type it
supports, and SHALL return an engine-agnostic report. Data of a type the engine
does not support SHALL be rejected with an error naming the types it accepts.

#### Scenario: Verifying through an engine

*Verification*: **executable** @engine @polars

- **GIVEN** the orders contract
- **AND** an engine constructed by the caller
- **AND** data of a type that engine understands
- **WHEN** the caller asks that engine to verify the contract against the data
- **THEN** a report is returned

#### Scenario: An engine rejects data it does not understand

*Verification*: **executable** @engine @polars

- **GIVEN** the orders contract
- **AND** an engine constructed by the caller
- **WHEN** the caller asks that engine to verify the contract against an object of an unsupported type
- **THEN** it fails with an error naming the data types that engine accepts

### Requirement: The caller names the engine, and the library never infers one

Verification SHALL be reached through an engine the caller constructs and names.
The library SHALL NOT infer an engine from the type of the data it is handed,
and SHALL NOT provide a registry or discovery mechanism that selects one. The
cost is accepted deliberately: the caller's own code is not framework-agnostic
even though the contract is, and auto-selection can be layered over explicit
engines later while the reverse is not recoverable.

#### Scenario: No dispatch path selects an engine

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the library's public surface
- **THEN** every path to verification requires the caller to name an engine
- **AND** no function inspects the data's type to choose one

### Requirement: The contract and the report are usable with no framework installed

Neither the contract model nor the report SHALL require any dataframe or compute
framework to be importable.

#### Scenario: Contract and report are usable with no framework installed

*Verification*: **executable**

- **GIVEN** no engine framework is importable
- **WHEN** the contract and report modules are imported
- **THEN** the import succeeds
- **AND** a contract can be constructed
- **AND** a report can be inspected

### Requirement: Exactly one module may reference each framework

For each engine, exactly one module SHALL import or reference that framework.
The contract model, the report, the shared engine base, and the package's public
exports SHALL NOT reference it — not in code, not in runtime-evaluated
annotations, and not in a default argument.

#### Scenario: Only the engine's own module names its framework

*Verification*: **non-executable**

- **WHEN** a reviewer inspects every module that imports or names a framework
- **THEN** only that framework's own engine module appears
- **AND** a change that appears to need the framework elsewhere is treated as
  evidence the boundary is wrong, rather than as a reason to import it

### Requirement: Each supported framework is an independent optional install

The base installation SHALL require no data framework. Each supported framework
SHALL be available as its own optional extra, and no extra SHALL require another
framework's package.

#### Scenario: The base install pulls in no framework

*Verification*: **executable**

- **WHEN** the library's declared runtime dependencies are inspected
- **THEN** they contain no dataframe or compute framework

#### Scenario: Each framework has its own extra

*Verification*: **executable**

- **WHEN** the library's declared optional extras are inspected
- **THEN** there is one extra per supported framework
- **AND** no extra requires another framework's package

### Requirement: An extra exists only once an engine exists behind it

An optional extra SHALL be added in the same change as the engine it installs,
and never in advance of one. An extra that installs a framework the library
cannot yet use is discoverable by users before it is true.

#### Scenario: No extra is published ahead of its engine

*Verification*: **non-executable**

- **WHEN** a reviewer compares the declared optional extras against the engines
  the library provides
- **THEN** every extra has a working engine behind it

### Requirement: Obtaining an engine whose framework is absent fails with an actionable error

Where an engine's framework is not installed, obtaining that engine SHALL fail
with an error naming the missing framework and the extra that installs it. It
SHALL NOT fail with an unqualified import error, SHALL NOT fall back to another
engine, and SHALL NOT return a partial or passing result.

#### Scenario: A missing framework names its extra

*Verification*: **executable**

- **GIVEN** the framework "polars" is not importable
- **WHEN** the caller constructs the Polars engine
- **THEN** it fails with an error naming the framework "polars"
- **AND** that error names the extra that installs it
- **AND** the failure is not a bare import error

### Requirement: The Polars engine verifies both eager and lazy in-memory frames

The library SHALL provide a Polars engine, installed by the `polars` extra, that
verifies a contract against an in-memory frame held by the calling process,
whether that frame is eager or lazy. The same contract and the same data SHALL
produce the same verdict and the same check outcomes through either.

#### Scenario: Verifying an eager frame

*Verification*: **executable** @engine @polars

- **GIVEN** the orders contract
- **AND** an eager Polars frame holding data for that contract
- **WHEN** the Polars engine verifies the contract against that frame
- **THEN** a report is returned covering every check the contract implies

#### Scenario: A lazy frame produces the same report as an eager one

*Verification*: **executable** @engine @polars

- **GIVEN** the orders contract
- **AND** an eager Polars frame holding data for that contract
- **AND** a lazy Polars frame holding the same data
- **WHEN** the Polars engine verifies the contract against each of them
- **THEN** both reports hold the same verdict
- **AND** both reports hold the same check outcomes
- **AND** both reports hold the same violating row counts
