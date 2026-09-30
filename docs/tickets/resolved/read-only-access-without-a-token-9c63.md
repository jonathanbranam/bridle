---
id: 9c63
title: Read-only bridle access for any local session, without a token
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What happened

The human started a plain Claude Code session in the clone and asked it to look
at completed bridle work. It couldn't: inside Claude Code the CLI never uses the
human's token, and with no `BRIDLE_TOKEN` it fails with "set `$BRIDLE_TOKEN`"
(docs/design/agent-host/principals.md, "How the CLI picks a token"), for reads
as well as writes.

The human, verbatim (2026-09-28):

> I started a separate agent and asked it to look at completed bridle work. it can't. So,
> that is a real issue; read-only bridle access shouldn't be a problem, IMO, for any agent.
> Also, that agent will need an actual token with provenance and the role needs a name.

## Why it matters

Reading tasks, agents, events and logs is harmless, and any session helping the
human needs it. Requiring a minted token just to read makes every ad hoc session
fail on its first `bridle` call.

## Notes

- Writes keep needing a token with provenance, as today. The human's second
  point is about that: a session that acts gets its own named token and role
  (for the advisor: `external:advisor`, `scripts/claude-advisor`).
- The daemon listens on 127.0.0.1 only.
- The human's standing stance: "I trust Claude agents so I don't think we need
  to go overboard in restricting their access too much." KISS.

## Resolution

`auth_middleware` (`crates/bridle-daemon/src/server.rs`) no longer 401s a
`GET`/`HEAD` request that has no bearer token: it authenticates as a new
synthetic `local` principal (`PrincipalKind::Local`,
`crates/bridle-api/src/types.rs`) instead, never stored, never minted. A
`GET` with a *valid* token still authenticates normally with real
attribution. Every `POST`/`PATCH`/`DELETE` route is unaffected and still
401s with no token. See [[docs/design/agent-host/principals#Read access
without a token|principals.md]] and `docs/design/agent-host/api.md`.

The human's second point — a session that acts getting a named token with
provenance — is out of scope here; that's the existing `external:advisor`
pattern (`bridle token create`), unchanged by this ticket.

Resolved 2026-09-28.
