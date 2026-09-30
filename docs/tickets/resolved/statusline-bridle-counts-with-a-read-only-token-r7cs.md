---
id: r7cs
title: Bridle counts in the status line, with a dedicated token
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [s8kn]
closed: 2026-09-30T05:12:44Z
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

`$BRIDLE_TOKEN` was the first idea, but it doesn't work: `resolve_token`
(`crates/bridle-api/src/discovery.rs`) checks `$BRIDLE_TOKEN` first,
unconditionally, for *every* command — the `$CLAUDECODE` gate only guards the
human-token-file fallback. Exporting `$BRIDLE_TOKEN` in the shell profile so
`statusline` could read it would make every other bridle command the human
runs (`stop-daemon`, `budget hold`/`override`, `token create`, ...) act as
that token's principal too, which is a real footgun the human had already
flagged once. The fix instead is a dedicated file,
`~/.bridle/statusline.token` (`discovery::statusline_token_path`), that only
`statusline` reads. Setup: `bridle token create statusline >
~/.bridle/statusline.token`. `statusline` makes its own best-effort,
2s-timeout `GET /v1/status` call when that file holds a token (and only
then — it never falls back to `$BRIDLE_TOKEN` or the workspace's human token
file, unlike other CLI commands), appending "N working · M for you" to the
line. Any failure (missing/empty file, no daemon, timeout, HTTP error) is
silent to the line, logged at `tracing::debug`.

The token is not scoped read-only or to this route: bridle has no
per-route/per-token scoping yet, so it can do whatever an `external:*`
principal can do. That's a known gap, not solved here. Documented in
[[docs/design/cli#Built|cli.md]].
