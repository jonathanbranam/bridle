---
id: g3az
title: "Status line token setup in the docs writes an empty file: token create needs --print now"
kind: chore
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [r7cs]
tasks: [br-g3az]
closed: 2026-10-09T23:11:03Z
---

## The ask

The human, verbatim (2026-10-09 ~7:45 AM ET), after the aide found dalek's status line had no bridle counts and suggested fixing the docs: "yes fix the docs."

## Facts (checked by the aide)

- `docs/design/cli.md` (the `bridle statusline` entry) and [[statusline-bridle-counts-with-a-read-only-token-r7cs|r7cs]]'s Resolution give the setup as `bridle token create statusline > ~/.bridle/statusline.token`.
- `bridle token create` now saves the token in `credentials.toml` and prints it only with `--print` (`bridle token create --help`). So that command writes an empty file: dalek's `~/.bridle/statusline.token` has been 0 bytes since 2026-09-30, and its status line shows no "N working · M for you".

## The ask

Fix the setup line wherever it appears (cli.md; the docs topic text in `docs/cli/` if it has one; `bridle statusline`'s help) to a command that works, e.g. `bridle token create statusline --print > ~/.bridle/statusline.token`. `--print` alone isn't enough: it also writes a `principal ...` line to stdout before the token (checked 2026-10-09: the human's file came out as two lines, and the status line failed silently). Either make the documented command keep only the token, or have `statusline` read the last non-empty line, or both. The resolved ticket r7cs stays as it is (a dated record).

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
