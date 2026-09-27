---
id: m9wt
title: "Spike: confirm Claude Code's statusLine stdin JSON schema"
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

`crates/bridle/src/statusline.rs` implements `bridle statusline`, which
Claude Code's `statusLine` setting invokes with a JSON document on stdin.
While building it, the exact schema of that document (field names, nesting,
which fields are optional) couldn't be confirmed: the worker's environment
had no web access (`WebFetch`/`WebSearch` both unavailable), and nothing in
this repo pins the schema down.

The code parses the input tolerantly and flags the uncertainty in its own
doc comments. Verify the real schema against Claude Code's own documentation
(wherever it's published) and check the field names `statusline.rs` expects
against it.

## Why it matters

If the field names differ from what `statusline.rs` assumes, the status
line silently renders with missing or wrong data instead of failing loudly
— worth confirming rather than leaving to tolerant parsing indefinitely.
`crates/bridle/src/statusline.rs` should be adjusted if the schema differs.
