---
id: 6hx4
title: Binaries accept unknown config sections and keys with a warning; doctor warns, and fails with a strictness flag
kind: feature
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [jmpf, 2ax5, 95mu]
tasks: [br-6hx4]
closed: 2026-10-09T23:11:05Z
---

## The ask

The human, 2026-10-06 ~6:35 PM ET (verbatim): "I think that's pretty annoying actually, I think binaries should be OK with unrecognized config sections and it makes upgrading far easier. They should WARN I think; and `doctor` should warn and/or fail or whatever. Maybe doctor should have an argument about whether warnings are failures or not."

## Today

`#[serde(deny_unknown_fields)]` on config structs: crates/bridle-daemon/src/config.rs (6 places), crates/bridle-api/src/machines.rs, crates/bridle-mail/src/config.rs. An older binary refuses a config.toml that has a section or key a newer one added, so editing config before upgrading stops the daemon (jmpf: a [gateway] section stopped every daemon). credentials.toml is already lenient.

## Decided by the human

- Unknown sections and keys are accepted, with a warning (logged by the daemon at start-up and config reload; printed to stderr by the CLI once per run, not on every command line of output), naming the file, the section or key, and "unknown to this build (bridle <version>); a newer build may use it".
- `bridle daemon doctor` reports unknown sections and keys as warnings. An argument makes warnings failures (e.g. `--strict`, exit non-zero), for CI or checks.

## To settle in the design (exact names before building, per 95mu)

- The flag's name, and whether doctor's default is warn or fail.
- Typos: a misspelt known key (`max_worker`) should still be caught; a "did you mean" suggestion when close to a known key.
- Which files: config.toml (machine and project), credentials.toml, machines, bridle-mail config, workflow.toml.
- Tests: an old-shaped struct reading a newer file.

Related: jmpf, 2ax5, 95mu.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
