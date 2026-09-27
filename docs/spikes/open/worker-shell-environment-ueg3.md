---
id: ueg3
title: Spike: controlled shell environment for workers
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups @ c192bfc:

> **Worker shell environment:** Bash sources the user's `~/.zshrc` via the snapshot. Decide whether
> workers should get a controlled `SHELL`/rc for reproducibility and startup time.

## Why it matters

Workers inherit whatever the human's shell does. On a separate host the rc
files differ again (bash on Ubuntu, zsh on the Mac).

## Notes
