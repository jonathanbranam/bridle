---
id: 6rh7
title: Shell completions for the bridle command (zsh and bash)
opened: 2026-09-29
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-10-09T23:11:02Z
---

## The ask

The human, verbatim (2026-09-29, via the advisor):

> add a ticket for zsh / bash completions for bridle command

## Today (at 0515f0d)

- The CLI is clap 4 with derive (`Cargo.toml`: `clap = { version = "4", features = ["derive",
  "env"] }`; `crates/bridle/src/cli.rs`). No completions exist anywhere in the repo.

## Shape

- `bridle completions <zsh|bash>` prints the script, generated from the clap definition with
  `clap_complete`, so it never drifts from the CLI. The human installs it once (e.g. `bridle
  completions zsh > ~/.zfunc/_bridle`); document that in `docs/design/cli.md`.
- Static completions (subcommands, flags, enum values like `--kind`) first. Dynamic ones (agent
  names, task IDs, project names from `~/.bridle/daemons/`) are a nice-to-have, only if cheap.
Work status: 2 integrated task(s); 2 dropped: br-146a, br-efda.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
