---
id: r7cs
title: Bridle counts in the status line, with a read-only token
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [s8kn]
---

## The question

The human asked for the status line, inside a bridle workspace, to show "some
indication of the work being done (counts) and waiting messages for me
(counts)" ([[statusline-real-context-and-a-tighter-layout-s8kn|s8kn]]). That
was cut from s8kn on 2026-09-28, along with the status line's recording.
Reading those counts from the daemon needs a token. Under Claude Code
`$CLAUDECODE` is set, so the CLI won't fall back to the human token file
(`docs/design/agent-host/principals.md`, "How the CLI picks a token"). What
read-only identity should the status line use, and how does the human set it
up with the least effort?

The human's words, 2026-09-28:

> Great. Yes let's add a later ticket for the read only bridle status.

## Why it matters

The human keeps an eye on bridle from whatever session is open. Counts in the
status line (agents working, messages waiting for the human) would show when
bridle needs them without running `bridle status`.

## Resolution

The `$CLAUDECODE` gate in `resolve_token`
(`crates/bridle-api/src/discovery.rs`) only guards the human-token-file
fallback; `$BRIDLE_TOKEN` itself is checked first, unconditionally. So a
human-minted `external:statusline` token (`bridle token create statusline`),
exported once as `$BRIDLE_TOKEN` in the shell profile, already flows through
normal token resolution — no new scoping or daemon endpoint needed.
`statusline` now makes its own best-effort, 2s-timeout `GET /v1/status` call
when `$BRIDLE_TOKEN` is set (and only then — it never falls back to the
workspace's human token file, unlike other CLI commands), appending "N
working · M for you" to the line. Any failure is silent to the line, logged
at `tracing::debug`. Documented in
[[docs/design/cli#Built|cli.md]].
