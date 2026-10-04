# Project resolution

How a bridle command learns which project it acts on. The shared resolver is
`crates/bridle/src/project.rs`; the daemon-reaching commands get the same order from
`bridle_api::discovery::resolve_endpoint`. See `docs/design/cli.md`, "Project resolution".

## Requirements

### Requirement: Every project-scoped command picks its project from the same sources, in order  {#r-f2c6}

Every command that acts on a project, subcommands included, SHALL take the project from
`--project`, then from `$BRIDLE_PROJECT`, then from the bridle workspace containing the
current folder, and SHALL use the first that is set.

#### Scenario: A command run inside a workspace acts on that workspace's project  {#s-a438}

*Verification*: **executable**

- **GIVEN** a workspace for project X, with no --project and no $BRIDLE_PROJECT
- **WHEN** any project-scoped command is run from a folder inside it
- **THEN** the command acts on project X

#### Scenario: The flag wins over the workspace  {#s-9bc0}

*Verification*: **executable**

- **GIVEN** a workspace for project X and a registered project Y
- **WHEN** any daemon-reaching command is run inside X's workspace with --project Y
- **THEN** the command acts on project Y

#### Scenario: The environment variable wins over the workspace  {#s-0bd0}

*Verification*: **executable**

- **GIVEN** a workspace for project X and a registered project Y
- **WHEN** any daemon-reaching command is run inside X's workspace with $BRIDLE_PROJECT set to Y
- **THEN** the command acts on project Y

### Requirement: A project-scoped command with no project refuses, and never assumes bridle  {#r-d8c2}

A project-scoped command that finds no project in any of those sources SHALL refuse and name
`--project`, and SHALL NOT fall back to a hard-coded project name. A command that creates
something for a repo not yet served (the launchd and systemd units, a new ticket) MAY name the
project after the repo's folder, but never after a constant.

#### Scenario: Outside any workspace the command refuses  {#s-f742}

*Verification*: **executable**

- **GIVEN** a folder outside every bridle workspace, with no --project and no $BRIDLE_PROJECT
- **WHEN** any daemon-reaching or session-launching command is run there
- **THEN** it exits with an error that names --project, and reaches no daemon

#### Scenario: A repo not yet served is named after its folder  {#s-c389}

*Verification*: **executable**

- **GIVEN** a git repo outside every bridle workspace, in a folder called fresh-repo
- **WHEN** daemon launchd install, daemon systemd install or ticket new is run there
- **THEN** the project it names is fresh-repo

### Requirement: A new command must declare how it finds its project  {#r-34c3}

Every command and subcommand SHALL be classified as reaching a daemon, launching a session,
naming a project, or acting on no project (with the reason), so a new one cannot ship unchecked.

#### Scenario: An unclassified command fails the check  {#s-4b92}

*Verification*: **executable**

- **GIVEN** the full tree of commands and subcommands in the bridle binary
- **WHEN** the project-resolution scenarios classify each of them
- **THEN** a command that fits no class fails the check, naming it
