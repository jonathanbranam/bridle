---
id: 6q8s
title: Replay echo matching by text
opened: 2026-09-27
repos: [bridle]
changes: []
specs: [docs/design/agent-host/messages.md]
needs: []
see: []
closed: 2026-09-30T05:12:44Z
---

## The question

From `docs/agent-host.md` §13 @ c192bfc, item 3:

> **`--replay-user-messages` echo matching by text** assumes claude echoes
> the text verbatim. It did in spike 01. If a future version adds a `uuid` to
> stdin user messages, match on that instead.

## Why it matters

Message delivery tracking depends on matching each echo to the message sent.

## Notes

Evidence: [spike 01 findings](docs/spikes/01-stream-json-findings.md).

## Resolution

Keep matching by text, oldest first, as built. It isn't a design choice to
revisit until claude changes its echo, and the guard against that change is a
contract test on upgrade:
[[pinning-and-checking-the-claude-code-version-enz3|version pinning]]. Recorded
in [[docs/design/agent-host/messages#Acks|messages and delivery]].
