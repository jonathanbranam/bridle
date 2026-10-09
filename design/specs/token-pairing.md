# Token pairing

How `bridle token pair` sets up tokens between machines over ssh, and keeps doing so as roles,
machines and projects are added. The code is `crates/bridle/src/token_pair.rs`; the design is
in `docs/tickets/open/pair-machines-token-setup-over-ssh-sk7p.md`. Part 1 covers role tokens;
peer tokens and the `[mail] peers` opt-out are added by br-jw9e.

## Requirements

### Requirement: All roles that need a token pairing are paired when the command runs.  {#r-d890}

`bridle token pair` SHALL pair every role in the one token-role list, for every selected
machine and project, and SHALL read that list rather than a copy of it. A launcher SHALL set
`BRIDLE_AS` only to a role in the list.

#### Scenario: Every listed role is paired everywhere  {#s-891b}

*Verification*: **executable**

- **GIVEN** two machines, each with a project daemon
- **WHEN** bridle token pair runs with no options
- **THEN** every machine holds a token for every listed role for every project on every machine, except that human is held only for the other machine's projects

#### Scenario: A role added to the list is paired by the next run  {#s-ac41}

*Verification*: **executable**

- **GIVEN** machines already paired, and a role newly added to the token-role list
- **WHEN** bridle token pair runs with no options
- **THEN** every selected machine has that role's token for every selected project, and no existing entry changed

#### Scenario: A launcher cannot sign as an unlisted role  {#s-d290}

*Verification*: **executable**

- **GIVEN** the sources of the session launchers and the mail service
- **WHEN** one sets BRIDLE_AS to a role that is not in the token-role list
- **THEN** a test fails

### Requirement: Every selector defaults to all, and naming some narrows to exactly those  {#r-a93e}

`--machines`, `--projects` and `--roles` SHALL each default to everything when left out, and
when given SHALL select exactly the names listed. There SHALL be no option that excludes.
`--roles` SHALL be an error when only peer tokens are selected.

#### Scenario: A named role, project and machine narrow the run  {#s-7504}

*Verification*: **executable**

- **GIVEN** machines with several projects and roles
- **WHEN** bridle token pair --projects notes --roles aide --tokens role runs
- **THEN** only the aide tokens for project notes are written

#### Scenario: Roles with peer tokens only is refused  {#s-9267}

*Verification*: **executable**

- **GIVEN** any machines
- **WHEN** bridle token pair --roles aide --tokens peer runs
- **THEN** it fails with a usage error and writes nothing

### Requirement: A second run changes nothing, and a failure never stops the rest  {#r-2af0}

An entry that works SHALL be kept. Only missing or broken entries SHALL be minted, and
`--rotate` SHALL replace working ones. An unreachable machine SHALL be reported and skipped,
the rest SHALL go on, and the exit status SHALL be non-zero. `--dry-run` SHALL mint and write
nothing.

#### Scenario: A second run keeps every entry  {#s-89c3}

*Verification*: **executable**

- **GIVEN** machines already paired
- **WHEN** bridle token pair runs again
- **THEN** every entry is kept and no credentials file changes

#### Scenario: An unreachable machine is skipped and reported  {#s-e443}

*Verification*: **executable**

- **GIVEN** one machine that ssh cannot reach
- **WHEN** bridle token pair runs
- **THEN** the reachable machines are paired, the failure is reported and the exit status is non-zero

#### Scenario: A dry run writes nothing  {#s-d9c0}

*Verification*: **executable**

- **GIVEN** machines not yet paired
- **WHEN** bridle token pair --dry-run runs
- **THEN** it lists what it would mint and mints and writes nothing

### Requirement: A token never appears in argv, logs or output  {#r-02e9}

A token SHALL travel only over helper stdout and stdin, in memory, and SHALL be written only
to the target machine's `credentials.toml`, mode 0600.

#### Scenario: No token is in a command line or in the output  {#s-f12f}

*Verification*: **executable**

- **GIVEN** a pairing run
- **WHEN** it finishes
- **THEN** no command it ran carried a token in its arguments and the output holds no token

### Requirement: Peer tokens let every project forward mail to every other, minted on the receiver  {#r-5d1c}

With `--tokens peer` (or by default) `bridle token pair` SHALL mint, on each selected receiving
project's daemon, `peer:<sending machine>`, and write it into the sending machine's
`credentials.toml` as `[peer] <receiving project>` (the direction rule, gdf3). Every selected
project that sends SHALL get one per receiving selected project, on the same machine too. An
entry present and not revoked SHALL be kept; `--rotate` and `--dry-run` apply as for role tokens.

#### Scenario: Peer tokens are minted on the receiver and stored on the sender  {#s-b3a1}

*Verification*: **executable**

- **GIVEN** projects bridle on machine mbp and notes on machine nuc
- **WHEN** bridle token pair --tokens peer runs
- **THEN** nuc mints a peer token for mbp and mbp holds a peer entry for notes, and the reverse; the output says peer minted on the receiver; a second run keeps both

### Requirement: A project can opt out of peer tokens  {#r-8e20}

`[mail] peers = false` in a project's `.bridle/config.toml` SHALL leave it out of peer tokens in
both directions. Its role tokens SHALL still be minted. The default is `true`.

#### Scenario: An opted-out project gets no peer tokens but keeps its role tokens  {#s-4e7d}

*Verification*: **executable**

- **GIVEN** a project with peers set to false in its mail config
- **WHEN** bridle token pair runs with no options
- **THEN** no machine holds a peer entry for it or sends from it, and its role tokens are minted

### Requirement: Creating a project pairs it  {#r-1f66}

`bridle init` SHALL run `bridle token pair --projects <name>` when the project already has a
daemon, and otherwise print the command as a next step after `bridle serve`. A failure SHALL
not fail init; it says to re-run the command. Existing projects default to `peers = true` and
get peer tokens on the next plain `bridle token pair`.

#### Scenario: Init names the pairing command  {#s-7a58}

*Verification*: **executable**

- **GIVEN** a new git repo
- **WHEN** bridle init runs
- **THEN** it succeeds and its next steps include the token pair command for the new project
