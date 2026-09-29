## Purpose

The declarative description of one flat table — its columns, their logical
types, their nullability, and the constraints on their values — held
independently of any data framework and of where, or whether, the data is
stored. This is the artifact that verification evaluates and that every later
capability attaches to.

## Requirements

### Requirement: A table contract declares a name and an ordered set of columns

A table contract SHALL declare a name that identifies it, and one or more
column declarations in a defined order.

#### Scenario: Constructing a table contract

*Verification*: **executable**

- **GIVEN** a column named "order_id" of type integer
- **AND** a column named "status" of type string
- **WHEN** a table contract named "orders" is constructed from those columns in that order
- **THEN** the contract reports the name "orders"
- **AND** the contract's column names are "order_id, status" in that order

#### Scenario: A contract with no columns is rejected

*Verification*: **executable**

- **WHEN** a table contract named "orders" is constructed with an empty column sequence
- **THEN** construction fails with an error stating that at least one column is required

#### Scenario: Duplicate column names are rejected

*Verification*: **executable**

- **GIVEN** two columns both named "order_id"
- **WHEN** a table contract is constructed from those columns
- **THEN** construction fails with an error naming the column "order_id"

### Requirement: A table contract describes a logical table only

A table contract SHALL declare no storage location, file format, partitioning,
or materialization, and SHALL be complete without them. The absence of storage
declarations SHALL NOT be reported as an omission.

#### Scenario: Storage is absent from the model, not merely optional

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the contract model
- **THEN** it exposes no location, file format, partitioning, or
  materialization declaration
- **AND** no code path treats their absence as an incomplete contract

### Requirement: A column declares a name, a logical type, and its nullability

Each column declaration SHALL carry a column name, exactly one logical type, and
whether null values are permitted in that column.

#### Scenario: Declaring a column

*Verification*: **executable**

- **WHEN** a column named "\<name\>" of type \<type\> is declared with nullable \<nullable\>
- **THEN** the declaration reports the name "\<name\>"
- **AND** the declaration reports the type \<type\>
- **AND** the declaration reports nullable \<nullable\>

*Examples*:

| name          | type    | nullable |
| ------------- | ------- | -------- |
| order_id      | integer | false    |
| unit_price    | float   | false    |
| customer_note | string  | true     |

### Requirement: The logical type system is owned by this library

Logical types SHALL be defined by the library. No dataframe or compute
framework's type object SHALL be accepted, stored, or returned anywhere in the
contract model, so that the choice of a first engine does not become a
permanent dependency of the contract format.

#### Scenario: No framework type appears in the contract model

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the contract model's declarations, annotations,
  and default values
- **THEN** none of them names a type belonging to a dataframe or compute
  framework
- **AND** mapping a framework's dtypes onto these logical types is the engine's
  responsibility

### Requirement: Exactly three logical types are available, and nothing else is accepted

A column declaration SHALL accept exactly three logical types — integer, float,
and string — and SHALL reject any other value.

#### Scenario: An unrecognised type is rejected

*Verification*: **executable**

- **WHEN** a column is declared with a value that is not one of the library's logical types
- **THEN** construction fails with an error naming the permitted types integer, float and string

### Requirement: Contracts are constructible with no data framework installed

Constructing and inspecting a contract SHALL succeed in an environment where no
engine framework is importable.

#### Scenario: A contract is built with no framework present

*Verification*: **executable**

- **GIVEN** no engine framework is importable
- **WHEN** a table contract is constructed and inspected
- **THEN** construction succeeds
- **AND** inspection succeeds

### Requirement: A numeric column may declare a range constraint

A column of integer or float type MAY declare a range constraint carrying a
lower bound, an upper bound, or both. A range constraint SHALL be rejected on a
column whose type is not numeric, SHALL be rejected when neither bound is given,
and SHALL be rejected when its lower bound exceeds its upper bound.

#### Scenario: Declaring a range with both bounds

*Verification*: **executable**

- **GIVEN** a column named "quantity" of type integer
- **WHEN** a range constraint with inclusive lower bound 1 and inclusive upper bound 999 is attached to it
- **THEN** the column reports a range constraint with inclusive lower bound 1 and inclusive upper bound 999

#### Scenario: Declaring an open-ended range with a single bound

*Verification*: **executable**

- **GIVEN** a column named "unit_price" of type float
- **WHEN** a range constraint with exclusive lower bound 0.0 and no upper bound is attached to it
- **THEN** the column reports a range constraint with exclusive lower bound 0.0 and no upper bound

#### Scenario: A range with no bounds is rejected

*Verification*: **executable**

- **WHEN** a range constraint is constructed with neither a lower nor an upper bound
- **THEN** construction fails with an error stating that at least one bound is required

#### Scenario: An inverted range is rejected

*Verification*: **executable**

- **WHEN** a range constraint is constructed with inclusive lower bound 10 and inclusive upper bound 1
- **THEN** construction fails with an error reporting the bounds 10 and 1

### Requirement: A bound carries its own inclusivity, and there is no default

A range bound SHALL be declared as a value together with whether that bound is
inclusive of its own value. Inclusivity SHALL have no default: a bound declared
without its inclusivity SHALL be rejected rather than treated as inclusive.
Inclusivity SHALL be carried by the bound itself, so that it cannot be declared
for a bound that is absent. The two bounds' inclusivity SHALL be independent, so
one range may be closed at one end and open at the other.

#### Scenario: A bound declared without its inclusivity is rejected

*Verification*: **executable**

- **WHEN** a bound is constructed with the value \<value\> and no inclusivity
- **THEN** construction fails with an error naming the undeclared inclusivity

*Examples*:

| value |
| ----- |
| 1     |
| 999   |

#### Scenario: Inclusivity cannot be declared apart from its bound

*Verification*: **non-executable**

- **WHEN** a reviewer inspects the range constraint's declaration
- **THEN** inclusivity appears only as part of a bound, never as a field of the
  range alongside it
- **AND** a range carrying an inclusivity for an absent bound is therefore
  unrepresentable rather than rejected at construction time

#### Scenario: The two bounds carry independent inclusivity

*Verification*: **executable**

- **GIVEN** a column named "discount" of type float
- **WHEN** a range constraint with inclusive lower bound 0.0 and exclusive upper bound 1.0 is attached to it
- **THEN** the column reports a range constraint with inclusive lower bound 0.0 and exclusive upper bound 1.0

### Requirement: A string column may declare a closed set of allowed values

A column of string type MAY declare an allowed-values constraint carrying a
non-empty set of permitted values. A value outside the set SHALL be a violation.
An allowed-values constraint SHALL be rejected when its value set is empty.

#### Scenario: Declaring allowed values

*Verification*: **executable**

- **GIVEN** a column named "status" of type string
- **WHEN** an allowed-values constraint of "pending, paid, shipped, delivered, cancelled" is attached to it
- **THEN** the column reports an allowed-values constraint of exactly those five values

#### Scenario: An empty allowed-values set is rejected

*Verification*: **executable**

- **WHEN** an allowed-values constraint is constructed with no values
- **THEN** construction fails with an error stating that at least one value is required

### Requirement: A constraint is rejected on a column of the wrong type family

A range constraint SHALL be rejected on a column that is not numeric, and an
allowed-values constraint SHALL be rejected on a column that is not a string.

#### Scenario: A constraint on the wrong type family is rejected

*Verification*: **executable**

- **GIVEN** a column named "\<column\>" of type \<type\>
- **WHEN** a \<constraint\> constraint is attached to it
- **THEN** construction fails with an error stating that \<constraint\> constraints apply only to \<family\> columns

*Examples*:

| column   | type    | constraint     | family  |
| -------- | ------- | -------------- | ------- |
| status   | string  | range          | numeric |
| quantity | integer | allowed-values | string  |

### Requirement: A column may declare no constraint

A column declaration carrying no constraint SHALL be complete, not
underspecified. Such a column constrains only its type and its nullability.

#### Scenario: A free-form column is complete

*Verification*: **executable**

- **WHEN** a column named "customer_note" of type string is declared with no constraint
- **THEN** construction succeeds
- **AND** the column reports that it carries no constraint
