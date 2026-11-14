---
id: br-fjar
title: reply-to from home orchestrator to a project's aide lands at the home daemon's unread aide
kind: bug
created: 2026-11-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

A reply (`bridle send --reply-to m-...`) from the home orchestrator to a message sent by another project's aide must reach that aide, not the home daemon's own `external:aide`.

Seen 2026-11-14 on dalek (orchestrator@dalek): bridle's aide sent m-0671 and m-0673 to the orchestrator on dalek-home. In dalek-home they show from plain `external:aide`, with no machine or project origin. The orchestrator's `--reply-to m-0671` (m-0672) and `--reply-to m-0673` (m-0676) went to dalek-home's own `external:aide`, which has no session, so they sat "arrived" and the bridle aide never got the heads-up it asked for before the firewall test. The orchestrator role says --reply-to always routes to the sender's daemon; here it didn't. Workaround used: `bridle send --project bridle external:aide`.

The human, 2026-11-14, via aide (m-0677), verbatim: "that's interesting on the addressing side; the orchestrator has no aide; I think I want to keep it that way". So: do not add an aide to dalek-home.

## Expected (either, the worker picks the simpler and says which on the thread)

1. A message cross-posted from another project's aide carries its origin (project, and machine where relevant), so `--reply-to` routes back to that project's aide; or
2. a reply to an `external:aide` that has no session on this daemon fails (or warns loudly) and prints the command that works (`bridle send --project <p> external:aide`), as the role doc says it should.

## Scope

Read first: how `--reply-to` resolves its target (crates/bridle/src/commands/misc.rs, cli.rs), how cross-project sends record the sender, and the orchestrator role text on --reply-to. Don't add an aide to dalek-home. Wire change only if option 1 needs it: then crates/bridle-api/src/types.rs, all clients and the daemon together, plus a migration note (none expected: old messages just lack origin). Docs: docs/design/cli.md where --reply-to is described, and the role doc if its claim changes.

## Acceptance

Tests: a reply to a cross-project aide message reaches that project's aide (option 1), or a reply to a sessionless aide errors with the working command (option 2); an ordinary same-project reply is unchanged. `just check` with BRIDLE_* env unset. Model: Sonnet.
