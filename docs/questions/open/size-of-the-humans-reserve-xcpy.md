---
id: xcpy
title: How big is the human's reserve?
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [3nqf, xypj]
---

## The question

From `docs/design.md` §15 @ c192bfc, item 13:

> **The human's reserve.** 25% of the 5-hour window is a guess. The right
> number depends on how much the human works interactively while workers run.

## Why it matters

Running out mid-conversation is the worst outcome
([[docs/design/usage-and-budget#The budget governor|budget governor]]).

## Notes

- The workforce may run on a separate machine
  ([[docs/context/nuc-host|the NUC]]). The reserve is per account, so it has to
  cover the human's sessions on every machine, including Remote Control from a
  phone.
