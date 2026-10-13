# Cart

## Requirements

### Requirement: Totals add up  {#r-a1b2}

The cart SHALL round once, on the total.

#### Scenario: Three items  {#s-c3d4}

*Verification*: **executable**

- **GIVEN** three items at 0.333
- **WHEN** the cart is totalled
- **THEN** the total is 1.00

#### Scenario: Reads well  {#s-e5f6}

*Verification*: **non-executable**

The total is shown to the cent.
