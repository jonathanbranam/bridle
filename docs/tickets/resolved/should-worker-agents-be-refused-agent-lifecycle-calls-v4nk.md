---
id: v4nk
title: Should worker-role agents be refused agent lifecycle API calls?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

A worker-role agent gets unrestricted Bash and its own valid `BRIDLE_TOKEN`
(a worker principal token). Nothing stops it from calling `bridle spawn`,
`bridle stop` or `bridle rm` on other agents via that token — the API has no
role check distinguishing a worker principal from a manager or orchestrator
principal for those endpoints.

Should worker principals be refused spawn/stop/rm on other agents, or is
that an acceptable trust boundary given they're already running arbitrary
shell commands (and could, in principle, forge the same calls by other
means)?

## Why it matters

This surfaced during bridle's first self-hosted run: a worker-role agent was
observed able to spawn/stop/rm other agents, which wasn't the intended shape
of the worker role as described in `docs/design/agent-host/roles-and-config.md`.
Whether this is a gap worth closing (a server-side role check on those
endpoints) or a non-issue (workers already have Bash, so an API restriction
adds no real containment) is unsettled.

## Resolution

Close the gap: worker principals are refused on the six agent lifecycle
endpoints (spawn, interrupt, stop, resume, renew, remove). `require_not_worker`
in `crates/bridle-daemon/src/server.rs`, next to the existing `require_human`
guard, looks up an `agent` principal's role via `state.store.get_agent()` and
returns `403` when it's `worker`; manager, orchestrator, human and external
principals are unaffected. Covered by
`worker_principal_is_refused_agent_lifecycle_endpoints` and
`manager_and_orchestrator_principals_keep_agent_lifecycle_authority` in
`crates/bridle-daemon/tests/lifecycle_test.rs`. Documented in
`docs/design/agent-host/principals.md`.

A worker with Bash can still forge the same HTTP calls by other means, so
this isn't a hard security boundary (see principals.md's provenance caveat),
but it does close the API-level gap and matches the intended shape of the
worker role.

Resolved 2026-09-28.
