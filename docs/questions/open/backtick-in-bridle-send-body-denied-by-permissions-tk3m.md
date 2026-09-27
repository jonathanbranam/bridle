---
id: tk3m
title: How should a bridle send/spawn body avoid the permission layer's backtick check?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

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
