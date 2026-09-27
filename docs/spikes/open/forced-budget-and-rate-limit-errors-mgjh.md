---
id: mgjh
title: Spike: force budget errors, retries and rate-limit rejections
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What to find out

From `docs/spikes/01-stream-json-findings.md`, Risks and follow-ups @ c192bfc:

> **Budget errors, API retries and rate-limit rejections** weren't observed. They need a forced test,
> e.g. `--max-budget-usd 0.001` for `error_max_budget_usd`.

## Why it matters

The [[docs/design/usage-and-budget#The budget governor|budget governor]] pauses
and resumes on these events. Their shapes are guessed until observed.

## Notes

- **The budget cap is done**: [spike 02 findings](docs/spikes/02-budget-cap-findings.md).
  What's left is API retries (`system/api_retry`) and a `rejected`
  rate-limit event, which can't be forced cheaply; capture them when they
  first happen in a transcript.
