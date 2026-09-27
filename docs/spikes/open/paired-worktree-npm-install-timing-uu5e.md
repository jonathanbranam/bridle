---
id: uu5e
title: Spike: paired worktree and npm install timing
opened: 2026-09-27
repos: [bridle, harness, track-web]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/research/01-agent-runtime.md` §8 @ c192bfc, item 7:

> **Paired worktree + `npm install` timing** for harness + track-web. This is
> research 09's still-open stage C.

## Why it matters

[[docs/design/worktrees-and-ports|Paired worktrees]] are how harness works at all.

## Notes

On the NUC (SATA SSD, 2 cores), install and build time per worktree will
decide how many workers are practical.
