# Principals and provenance

## Principals

| Kind | Example id | Gets its token from |
|---|---|---|
| `human` | `human` | created on first start, `.bridle/tokens/human` (0600) |
| `agent` | `agent:w1` | minted at spawn, injected as `BRIDLE_TOKEN`, revoked at `rm` |
| `external` | `external:orchestrator` | `bridle token create orchestrator`, printed once and stored hashed (409 if the name is taken; no list or revoke yet) |
| `system` | `system` | bridle itself; no token |

Tokens are 64 hex characters, stored as SHA-256 hashes. If the human token
file goes missing, the next start revokes the old token and mints a new one.

Every event records its `actor`: the caller for spawn, send, read,
interrupt, stop, resume and remove, and `system` for what agents do and for
the state changes that follow. `token create` isn't recorded yet. Commands that are only
the human's (`token create`, `shutdown`, and later `accept`) refuse other
principals. A message can be marked read by its recipient or by the human.
Agent lifecycle endpoints (spawn, interrupt, stop, resume, renew, remove)
refuse an `agent` principal whose role is `worker`: the worker role has no
lifecycle authority ([[docs/design/agent-host/roles-and-config|roles and
config]]), even over other agents it didn't spawn itself. Manager,
orchestrator, human and external principals are unaffected.

An agent's token is also kept in `.bridle/agents/<id>/token` (0600), so
`resume` can re-inject the same identity; the store keeps only hashes.

## How the CLI picks a token

1. `--token`, then `$BRIDLE_TOKEN`.
2. Otherwise, **only if `$CLAUDECODE` is unset**, the human token file.
3. Otherwise, fail with "set `BRIDLE_TOKEN`". This is also what happens when
   the daemon was found by URL, since there is no local workspace to hold a
   human token file.

Rule 2 means a Claude Code session (the human's orchestrator, or any agent)
never silently acts as the human. It has to be given an identity.

## What this is and isn't

On one machine as one user, a token file is readable by any process of that
user, so provenance is **attribution that honest agents can't get wrong by
accident, not a security boundary**. Two things keep it honest:

- The human token is never passed into an agent's environment.
- Agent worktrees don't contain any token.

Once bridle listens on a non-loopback interface, the tokens become real
authentication. Remote deployments should also put bridle behind SSH port
forwarding, a VPN or TLS termination ([[docs/design/agent-host/api#Running the workforce remotely|running remotely]]).
Whether provenance should be stronger is open:
[[how-strong-agent-provenance-should-be-2bzw|how strong provenance should be]].
