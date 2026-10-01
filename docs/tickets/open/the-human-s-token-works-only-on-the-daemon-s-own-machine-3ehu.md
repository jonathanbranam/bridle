---
id: 3ehu
title: The human's token works only on the daemon's own machine
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [k7mw, cw7a]
---

## The ask


The human, 2026-10-01, from the NUC (relayed verbatim by the NUC's meta-notes orchestrator,
m-3001 on bridle's daemon):

> (2) An issue the human wants resolved: the human can't send bridle messages from nuc to dalek.
> On nuc, 'bridle send --project bridle advisor "text"' fails with 'error: no workspace found to
> read the human token from: set $BRIDLE_TOKEN'. The human CLI reads its token only from
> <workspace>/.bridle/tokens/human, which exists only on the daemon's own machine, so a human
> token is in effect single-machine. The orchestrators and advisors have tokens for both machines
> (~/.bridle/credentials.toml, e.g. [orchestrator.dalek]); the human created and shared those and
> should have the same cross-machine ability, without copying token files over ssh. Please file or
> fold this into a bridle ticket. (Side note: the human addressed 'advisor'; the address is
> external:advisor. Bridle could accept the bare role name or give a clearer error.)

## What's there now

`docs/design/agent-host/principals.md`, "How the CLI picks a token" (at 54b339a): `--token`,
`$BRIDLE_TOKEN`, then `$BRIDLE_AS`'s entry in `~/.bridle/credentials.toml` (the
`[<principal>.<machine>]` sub-table for a project on another machine, k7mw), then, outside
Claude Code, the workspace's human token file. A project on another machine resolves with no
workspace (`resolve_endpoint_with`, `crates/bridle-api/src/discovery.rs`), so with neither
`--token`, `$BRIDLE_TOKEN` nor `$BRIDLE_AS` set, the human's plain command has no token to use.

`resolve_token_in` doesn't restrict `$BRIDLE_AS` to external principals, so
`[human.dalek] bridle = "..."` plus `BRIDLE_AS=human` might already work, but nobody has tried
it. That still means pasting the token into a file by hand, once per machine and project, the
same way the orchestrator and advisor tokens were set up.

Two asks: the human's plain commands reach a project on another machine, the way the agents'
do; and a bare role name such as `advisor` as a `send` recipient is either accepted or
rejected with an error that names `external:advisor`.
