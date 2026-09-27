---
id: r5hc
title: When is chunking a message to a manager worth its overhead?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

The manager role is expensive to run — bridle's first self-hosted run spent
several dollars across a handful of turns on Sonnet. Part of that cost came
from sending a long brief to the manager as several separate chunked
messages, each of which started its own new turn on an otherwise-idle agent
(per the idle-message behavior in `docs/spikes/01-stream-json-findings.md`),
multiplying turn-start overhead compared to sending the whole brief as one
message.

When is chunking a message worth its overhead, versus sending one larger
message? Is there a size or structural threshold past which chunking stops
paying for itself given bridle's per-turn cost?

## Why it matters

This is purely a cost/design question about how bridle-driving-bridle
sessions (or humans) should compose messages to a manager, not a bug: each
chunk is a legitimate message, but the turn-per-chunk cost adds up on an
expensive role.
