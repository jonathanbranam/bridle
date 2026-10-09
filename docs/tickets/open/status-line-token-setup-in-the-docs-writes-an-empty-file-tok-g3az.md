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
tasks: []
---

## The ask

The human, verbatim (2026-10-09 ~7:45 AM ET), after the aide found dalek's status line had no bridle counts and suggested fixing the docs: "yes fix the docs."

## Facts (checked by the aide)

- `docs/design/cli.md` (the `bridle statusline` entry) and [[statusline-bridle-counts-with-a-read-only-token-r7cs|r7cs]]'s Resolution give the setup as `bridle token create statusline > ~/.bridle/statusline.token`.
- `bridle token create` now saves the token in `credentials.toml` and prints it only with `--print` (`bridle token create --help`). So that command writes an empty file: dalek's `~/.bridle/statusline.token` has been 0 bytes since 2026-09-30, and its status line shows no "N working · M for you".

## The ask

Fix the setup line wherever it appears (cli.md; the docs topic text in `docs/cli/` if it has one; `bridle statusline`'s help) to a command that works, e.g. `bridle token create statusline --print > ~/.bridle/statusline.token`. Check that `--print` writes only the token to stdout. The resolved ticket r7cs stays as it is (a dated record).
