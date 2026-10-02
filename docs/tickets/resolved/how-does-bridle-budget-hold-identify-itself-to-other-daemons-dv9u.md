---
id: dv9u
title: How does `bridle budget hold` identify itself to other projects' daemons?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [xypj]
closed: 2026-10-02T00:43:36.146315Z
---

## The question

`docs/design/usage-and-budget.md`, The human's hold, says `hold` "applies to
every daemon in the registry": the human wants the whole account idle, not
just the project they happen to be sitting in. `bridle-api::discovery::list_registry`
already gives the CLI every other project's daemon URL on this machine, so
reaching them isn't the hard part.

What's unsettled is how the CLI authenticates to a daemon it wasn't given a
token for. Every other command resolves a token for *its own* project
(`discovery::resolve_token`, per `docs/design/agent-host/principals.md`); a
cross-project `hold` needs some way to either:

- read every other project's human token off disk (they're all on the same
  machine, under each project's own `.bridle/tokens/human`), and hope the
  calling user has read access to all of them; or
- a new "hold" endpoint that accepts a different, machine-scoped credential
  instead of a per-project human token; or
- a local-only trust boundary (loopback + no token) for this one call, since
  every daemon already binds to `127.0.0.1`.

## Why it matters

Without this, `bridle budget hold`/`release` only take effect on the current
daemon. The wind-down build (crates/bridle-daemon/src/governor.rs) does
exactly that for now, and calls it out rather than guessing at a
cross-project identification scheme.

## Notes

- Relates to but is distinct from [[how-project-daemons-share-one-budget-xypj|how
  project daemons share one budget]]: that's about `max_workers` and dispatch
  rate; this is purely about the CLI-to-daemon credential for one
  human-only, machine-wide action.
- Whatever is decided should probably also cover `bridle daemons`, which
  already lists every registry entry without touching them.

## Resolution

Resolved by: br-9e4e (cc893d0)
