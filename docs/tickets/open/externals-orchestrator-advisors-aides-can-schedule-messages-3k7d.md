---
id: 3k7d
title: Externals (orchestrator, advisors, aides) can schedule messages for themselves
kind: bug
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [yfv5, cbbn]
tasks: []
theme: agents-and-cli
---

## The ask

The human, 2026-10-11 ~12:30 AM ET, verbatim (to advisor product-manager), on why externals get a
403 from `bridle schedule add`:

> yeah, those internal agents barely need it; the purpose of this is specifically for the extern
> agent. Yes, create a ticket and fix that and get it done tonight.

This is yfv5 Point 4, option B, now decided.

## Why it is broken

br-9xze's brief (decision 6, approved 2026-10-08) said "an agent may add a schedule only with itself
as the target ... The human may add for any target ... Everyone else refused." It only thought of
spawned agents and the human, so `external` principals (orchestrator, advisors, aides: the
long-lived interactive sessions the feature is for) fell into "everyone else".
`schedule_actor_ok` in `crates/bridle-daemon/src/server.rs` (~line 1726) allows only
`Human | Agent` and answers 403 "only agents and the human use schedules". The worker flagged it on
landing; slice 2 (br-g5y2) did not fix it.

## What to build

- Externals follow the agent rule: an external may add a schedule only with itself as the target
  (`--to` defaulting to itself), and list/rm only its own. The human keeps "any".
- Identity compare: a named advisor is `external:advisor/<name>`; it schedules for exactly that
  identity. Identities are shared (every project's aide is `external:aide`), so two sessions of one
  identity see each other's schedules; acceptable, they are the same human's sessions.
- `bridle schedule add` with no `--to` works for an external (defaults to the caller).
- Update the role text that tells sessions how to get a timed wake-up (advisor, aide, orchestrator
  roles in `workflow/base/roles/`, where g5y2 added it for agents) so externals know they can.
- Docs: daemon.md "Scheduled messages" (auth), api.md, cli.md, CHANGELOG.
- Tests: an external adds for itself (ok), for another principal (403), lists and removes only its
  own; the human still sees all.

Priority high: the human wants it landed tonight (2026-10-11). Small.
