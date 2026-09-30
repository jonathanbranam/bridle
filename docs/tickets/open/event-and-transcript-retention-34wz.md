---
id: 34wz
title: Event and transcript retention
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The question

From `docs/agent-host.md` §13 @ c192bfc, item 5:

> **Event retention** of 30 days is a guess. Transcripts grow without bound
> until `rm`.

## Why it matters

Disk on a small always-on host is finite ([[docs/context/nuc-host|the NUC]]
has a 500 GB SSD).

## Notes
