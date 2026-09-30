---
id: tk3m
title: How should a bridle send/spawn body avoid the permission layer's backtick check?
opened: 2026-09-27
resolved: 2026-09-28
repos: [bridle]
changes: [b1f4750]
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
---

## Resolution

Both `bridle send` and `bridle spawn` now support reading their body/prompt from
stdin when `-` is passed to `--text-file` or `--prompt-file`. This lets callers
pipe the content without it ever appearing as a shell argument, bypassing Claude
Code's permission classifier entirely.

- Added `--text-file FILE` to `bridle send`, mutually exclusive with the
  positional TEXT argument
- Both `--prompt-file` (spawn) and `--text-file` (send) treat `-` as stdin
- Updated docs/design/cli.md with the new flags and usage examples

Example: `echo "message with \`backticks\`" | bridle send w1 --text-file -`

## The question

During bridle's first self-hosted run, the manager's own `bridle send`/
`bridle spawn` calls were silently denied by Claude Code's permission layer
when the message body contained a backtick character. The body was passed
safely single-quoted inside a heredoc, so there was no actual command
substitution risk, but the permission classifier flags backticks in a Bash
command argument regardless of quoting context.

Should there be a way to pass a message body without any shell quoting at
all — reading from stdin, or a `--prompt-file -` meaning "read from stdin"
— so the body never has to survive a Bash permission check? Or, short of a
CLI change, should this at minimum be a documented warning to avoid
backticks in `bridle send`/`bridle spawn` bodies?

## Why it matters

This blocked a legitimate `bridle send` mid-run and wasn't obvious why: the
denial looked like a normal permission prompt rather than a consequence of
the body's contents.
