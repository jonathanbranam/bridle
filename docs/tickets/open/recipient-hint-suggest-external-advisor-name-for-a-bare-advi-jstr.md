---
id: jstr
title: "Recipient hint: suggest external:advisor/<name> for a bare advisor/<name>"
kind: bug
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [3k7d]
tasks: [br-jstr]
theme: agents-and-cli
---

## The ask

The human, 2026-10-11 ~1:00 AM ET, verbatim (to advisor product-manager):

> it doesn't suggest advisor though, when I tried 'aide' is suggested 'external:aide'. File a small
> ticket to fix the suggestion to strip the '/' when comparing to agents.

## What happens

`bridle send` / `bridle schedule add --to advisor/product-manager` fails with
`not_found: no such recipient: advisor/product-manager` and no hint. `--to aide` fails with the hint
"did you mean external:aide?". The hint comes from `is_known_external_principal` in
`crates/bridle-daemon/src/server.rs` (~line 1496), which compares the whole string, so
`advisor/product-manager` never matches `advisor`.

## Fix

Compare only the role part: strip a `/<name>` suffix (and an `@<machine>` suffix) before the match,
so `advisor/product-manager` suggests `external:advisor/product-manager`. The hint keeps the full
name the caller typed. Extend the existing unit test (`is_known_external_principal_recognizes_valid_names`)
with `advisor/product-manager` and `aide@nuc`.

Note: ticket [[externals-orchestrator-advisors-aides-can-schedule-messages-3k7d|3k7d]] (br-3k7d) also
makes bare known external names resolve outright. Whichever lands second keeps the hint for the
cases 3k7d doesn't resolve (an unknown name after `advisor/`), or drops it if nothing is left; say
which on the thread.
