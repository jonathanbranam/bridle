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
