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
`external:advisor/<name>` lands in that principal's inbox whether or not the session is registered
(`POST /v1/sessions`): unread mail stays there when the session ends, and the next session of that
name reads it. The send response carries `recipient_note` ("<name> isn't running; waiting in its
inbox") when no session of that name runs. An `@machine` address, which the local registry can't
vouch for, goes to `external:advisor` with "(originally for advisor/<name>)" put before the body.
Registering a named session moves any unread mail so marked in the shared inbox to its own inbox
and takes the mark off (the recovery for mail the earlier move-on-end stranded; hwek). An unknown
owner before the `/` is a 404.

Every event records its `actor`: the caller for spawn, send, read,
interrupt, stop, resume and remove, and `system` for what agents do and for
the state changes that follow. `token create` isn't recorded yet. Commands that are only
the human's (`token create`/`revoke`, `shutdown`, `rebuild`, and the `budget` writes: hold, release,
override, max-workers) refuse other principals; the queue writes (`POST /v1/queue`, `/queue/tiers`)
take only the human or an agent whose role is `project-manager`. A message can be marked read by its recipient or by the human.
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
   Exception, and only where rules 2-4 would end in the "no workspace found" error: the
   daemon is on another machine (`--project` routed by the machine config) and the
   credentials file has `[human.<machine>] <project> = "..."`. That token is used, as the
   principal `human@<machine>`; `$BRIDLE_AS=human` reads the same entry already. A local
   project always uses the workspace token file, whatever `[human.*]` holds.
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

`bridle token pair` (human only; spec `design/specs/token-pairing.md`) does all of this for every
listed role, project and machine at once over ssh, including the `[<role>.<machine>]` and
`human@<machine>` entries; it keeps entries that work, so it is safe to re-run. The roles it pairs
are the one list `TOKEN_ROLES` (`crates/bridle/src/token_pair.rs`), which the session launchers
also use; a launcher cannot set `BRIDLE_AS` to a role not in it. Agents' deny lists carry
`Bash(bridle token *)`.

A principal named `<name>@<machine>` is a **visitor**: another machine's principal on this
daemon, minted with `bridle token create <name> --machine <machine>` (printed once, to paste into
that machine's `credentials.toml`; plain `token create` refuses `@` in a name, so the suffix always
means a visitor). It sends, reads its own inbox and queries like any external principal. The
daemon's own `external:orchestrator` (wake long poll, liveness watch, handovers) is matched by
exact name, so a visitor never takes those over. The name `human` with `--machine` mints `human@<machine>` instead: the human on that machine, kind
human, so every human-only route (`token create`/`revoke`, `shutdown`, ...) accepts it exactly as
the local human; recorded and revocable separately (`token revoke human@<machine>`). Its messages
are from `human@<machine>`, `--to me` is its own inbox, and a reply to one of its messages (or
`--to human@<machine>`) lands there, not in the bare `human`'s inbox. `bridle session orchestrator`, `bridle session advisor` and `bridle session aide` (`external:aide`, `[aide]` in `credentials.toml`) set
`BRIDLE_AS` so a session never handles a token.

A visitor may also **submit** (`POST /v1/tasks/submit`, `bridle ticket submit`, ticket 93xm): an `open` task whose body starts
`submitted by <principal>`, with a first thread note saying so. It may comment on its own submissions only; planning,
claiming, dropping and editing a task are refused for any visitor.

## Mail between daemons (3haz, slice 1)

A message to a principal on another daemon (`bridle send --project <other> ...`, any machine, the
same machine included) never goes from the CLI to that daemon. It goes to the **sender's own**
daemon (`POST /v1/outbox`), which accepts it at once and answers with an outbox id, stores it in
its `outbox` table (storage.md) and forwards it to the destination over `POST /v1/forward`. The
destination is found from the machine config (`[projects]`, k7mw), else this machine's registry.
Exception (2msq): when the sender holds its own credential for that project (`$BRIDLE_AS`'s entry in
the credentials file) and the project's daemon is on this machine, `send` talks to it directly with that
token, like `task ready --project`; the outbox is for a project with no such token and for other machines.

- **Peer tokens.** `bridle token create --peer <machine>` on the *receiving* daemon mints
  `peer:<machine>` (`<machine>` is the sender's `[machine] name`; `local` when it has none).
  It is printed once and pasted into the sender's `credentials.toml` under `[peer]`, keyed by the
  receiving project: `[peer]` / `beta = "..."`. One token per sending machine per receiving
  daemon, so the daemons of one machine share theirs. A peer token may call `/v1/forward` and
  nothing else (403); nothing else may call `/v1/forward`.
- **The sender's label.** The forwarding daemon states who the sender was, qualified with its
  machine (`agent:w1@nuc`, `external:advisor/research@nuc`), and the receiver stores that as
  `from`. It is believed because the token is a peer's: a visitor's or other external token's
  label is only a label.
- **Replies come home (br-n7cg, 3haz P2).** `bridle token create <name> --machine <m> --home
  <project>` records the visitor's home: a project this daemon can forward to (machine config and
  a `[peer]` token, checked when the token is minted). Mail to that visitor, from `send` or a
  forward, goes into this daemon's outbox for the home project, addressed to the visitor's bare
  principal (`external:aide@nuc` becomes `external:aide`, `human@nuc` becomes `human`), and
  lands in the home daemon's ordinary inbox, where its own waiter wakes; nobody reads another
  machine. The sender is labelled with this machine (`external:advisor@dalek`). The `send` answers
  with the outbox id as the message id. Migration: a visitor token minted before this has no
  home (`principals.home` NULL) and keeps its inbox here as before; so does one whose home can no
  longer be forwarded to (no peer token), with a warning in the log. A home is only for visitors.
  Every principal's home is its own machine's daemon, so a visitor token with a home is the
  only record needed; `external:orchestrator` itself stays local.
- **Exactly once, in order.** Each forward carries its origin (machine, daemon, outbox id); the
  receiver records it in `forwarded_in` and answers a repeat with the same message ids without
  delivering again, so a try whose acknowledgement was lost is safe to repeat. Per destination
  the outbox delivers oldest first, one flush at a time; a try that doesn't get through (daemon
  down, refused token) leaves the message queued and stops the flush so nothing overtakes it;
  a refusal for good (unknown recipient, bad request) marks it `failed` and the queue moves on.
- **Retry (br-fvkq).** Backoff at once, 30 s, 2 m, then every 5 m; mail never expires; a start-up
  or wake-from-sleep greeting (`POST /v1/hello`, peer token) makes the hearer flush at once. See
  daemon.md, "Outbox retry". The send waits up to 3 s for the first try and answers with its
  outcome (`delivered`, `failed` with the reason, or `queued` with the last error). A refusal for
  good, and a message queued 30 min, each send the sender one note from `system`. The receiver
  accepts `agent:<name>` as well as a bare agent name.
- **Visible state (br-cufw, P6, P7).** `bridle status` has an `outbox <project> N queued,
  unreachable <age> (<last error>)` line per destination with mail queued (`Status.outbox`).
  `bridle message show <id>` reports a stage: `queued` (in the sender's outbox), `arrived` (stored
  on the recipient's daemon, recipient not yet woken: the message there is pending, held or
  written), `delivered` (the recipient received it, which for now is also read; Q4), or `failed`.
  For an `o-` id the sender's daemon (`GET /v1/outbox/{id}`, sender or human only) asks the
  destination (`GET /v1/forward/{message_id}`, peer token) each time; a destination it can't
  reach leaves it `arrived` with the error. A message queued over an hour (`[messages]
  undelivered_report_mins`, default 60) is also reported once to `external:aide` ("For the
  human: ..."), on top of the sender's 30 min note. `bridle recipients` (`GET /v1/recipients`)
  lists what can be addressed here (human, agents, external principals, visitors) and the other
  daemons known from the registry and machine config.
- **Not built yet:** `--task` and `@machine` addressing across daemons.

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
