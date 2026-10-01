# Principals and provenance

## Principals

| Kind | Example id | Gets its token from |
|---|---|---|
| `human` | `human` | created on first start, `.bridle/tokens/human` (0600) |
| `agent` | `agent:w1` | minted at spawn, injected as `BRIDLE_TOKEN`, revoked at `rm` |
| `external` | `external:orchestrator` | `bridle token create orchestrator`, stored hashed (409 if the name is taken); the CLI saves it in `~/.bridle/credentials.toml` (below) |
| `system` | `system` | bridle itself; no token |
| `local` | `local` | synthesized per-request for a `GET`/`HEAD` with no bearer token; never stored, never minted |

Tokens are 64 hex characters, stored as SHA-256 hashes. If the human token
file goes missing, the next start revokes the old token and mints a new one.

## Read access without a token

The daemon listens on 127.0.0.1 by default
([[docs/tickets/open/read-only-access-without-a-token-9c63|ticket 9c63]]):
any process on the machine can already reach it, so requiring a token just to
read is friction without a security benefit. That holds only for a peer on this
machine: the daemon grants `local` only when the TCP peer address is loopback
(set by the handshake, so a client can't spoof it), and any token-less request
from another address is a 401, reads included. Forwarding from this machine (an
`ssh -L` tunnel) arrives from loopback and counts as local. Starting with a
`listen` address that isn't loopback logs a warning. A `GET`/`HEAD` request from
loopback with no
`Authorization` header authenticates as a synthetic `local` principal instead
of 401ing. `local` never passes `require_human` or the worker lifecycle gate,
but it can't reach those anyway — both only guard
`POST`/`PATCH`/`DELETE` routes, which still 401 with no token. A request that
*does* present a token, valid or not, is checked as before: a valid token
authenticates normally (with its real principal and attribution) even on a
`GET`, and an invalid one is still a 401.

This is for read-only ad hoc sessions (a plain Claude Code session in the
clone, poking around with `bridle status`/`bridle agents`) that have no
`BRIDLE_TOKEN` per the rule below. That's only useful if the `bridle` CLI
itself cooperates: a read-only command run with `$CLAUDECODE` set and no
token sends the request anonymously (see "How the CLI picks a token" below)
instead of erring out client-side before any request is sent. A session that
needs to *act* still wants a named token with provenance, minted the way the
advisor's is (`external:advisor`, `bridle token create`) — this doesn't
replace that.

**Named advisors.** A named advisor shares `external:advisor`'s token; its CLI adds the name
(`BRIDLE_ADVISOR_NAME`, header `x-bridle-advisor`) and the daemon signs its requests
`external:advisor/<name>` (`@machine` kept): an honest label, not proof. Mail to
`external:advisor/<name>` lands in that principal's inbox while the session is registered
(`POST /v1/sessions`); otherwise (ended, never existed, or an `@machine` address, which the local
registry can't vouch for) it goes to `external:advisor` with "(originally for advisor/<name>)" put
before the body, and unread mail moves there the same way when the session ends. An unknown owner
before the `/` is a 404.

Every event records its `actor`: the caller for spawn, send, read,
interrupt, stop, resume and remove, and `system` for what agents do and for
the state changes that follow. `token create` isn't recorded yet. Commands that are only
the human's (`token create`/`revoke`, `shutdown`, `rebuild`, and the `budget` writes: hold, release,
override, max-workers) refuse other principals; the queue writes (`POST /v1/queue`, `/queue/tiers`)
take only the human or an agent whose role is `product-manager`. A message can be marked read by its recipient or by the human.
Agent lifecycle endpoints (spawn, interrupt, stop, resume, renew, remove)
refuse an `agent` principal whose role is `worker`: the worker role has no
lifecycle authority ([[docs/design/agent-host/roles-and-config|roles and
config]]), even over other agents it didn't spawn itself. Manager,
orchestrator, human and external principals are unaffected.

An agent's token is also kept in `.bridle/agents/<id>/token` (0600), so
`resume` can re-inject the same identity; the store keeps only hashes.

## How the CLI picks a token

1. `--token`, then `$BRIDLE_TOKEN`, then, if `$BRIDLE_AS=<principal>` is set, that
   principal's entry for the project the command talks to (`--project`, or the cwd's
   daemon) in `~/.bridle/credentials.toml`. A missing entry, or no known project (the
   daemon was found by URL), is an error naming the file, principal and project; it
   never falls through to the rules below.
2. Otherwise, **only if `$CLAUDECODE` is unset**, the human token file.
3. Otherwise, for a read-only command (`status`, `agents`, `show`, `logs`,
   `events`, `usage`, `inbox`, `task show`/`list`, `token list`, `budget`
   with no subcommand, `ready`), send the request with no token at all: the
   daemon's own tolerance for a token-less loopback `GET`/`HEAD` (above) then
   authenticates it as `local`; a daemon on another machine answers 401. The human token file is still never read
   implicitly under `$CLAUDECODE` — the CLI just stops erring out ahead of a
   request that would have succeeded anyway.
4. Otherwise (a write, or a non-read command that can't reach a workspace to
   find a human token file — e.g. the daemon was found by URL), fail with
   "set `BRIDLE_TOKEN`".

### The credentials file

`~/.bridle/credentials.toml` (`$BRIDLE_HOME/credentials.toml`), mode 0600, one table per
external principal and one key per project (names as in the registry):

```toml
[orchestrator]
bridle = "..."
track-web = "..."

[advisor]
bridle = "..."
```

A sub-table `[<principal>.<machine>]` holds the tokens for that machine's daemons; a plain
`[<principal>]` key is a project on this machine, so files written before machines existed
keep working. The CLI uses the sub-table when the machine config (below) puts the
project's daemon on another machine (k7mw). The human pastes those in by hand:

```toml
[advisor.nuc]
meta-notes = "..."
```

`bridle token create <name> --project <p>` adds `[name] p = token` (creating the
directory and file 0600, keeping other entries) and prints no token; with no known project
(`--url`) it prints the token as before. `bridle token revoke <name> --project <p>`
removes the entry. The CLI refuses to read a file that group or others can access, and
says to `chmod 600` it. `--print` prints the token as well.

A principal named `<name>@<machine>` is a **visitor**: another machine's principal on this
daemon, minted with `bridle token create <name> --machine <machine>` (printed once, to paste into
that machine's `credentials.toml`; plain `token create` refuses `@` in a name, so the suffix always
means a visitor). It sends, reads its own inbox and queries like any external principal. The
daemon's own `external:orchestrator` (wake long poll, liveness watch, handovers) is matched by
exact name, so a visitor never takes those over. `bridle session orchestrator` and `bridle session advisor` set
`BRIDLE_AS` so a session never handles a token.

Rule 2 means a Claude Code session (the human's orchestrator, or any agent)
never silently acts as the human. It has to be given an identity to write;
rule 3 only ever gets it as far as `local` can reach, which is reads.

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
