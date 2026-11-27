---
id: br-ysmy
title: Daemons fetching a release workflow at once share one scratch dir and can fail
kind: bug
created: 2026-11-27
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Several daemons on one machine fetching the same release workflow at once must not break each other.

Found by the worker on br-kf7a (landed 2406b3fb), not fixed there: `workflow_checkout::ensure` uses a shared scratch dir `~/.bridle/workflow/.fetching-<tag>`, so concurrent daemons (every one on a machine after an upgrade) can fail the same way as the shared `<exe>.new` file did. Non-fatal: it is retried at the next start.

## Suggested fix

A scratch dir unique per process, then rename into place; if the target already exists (another daemon won), treat that as success and remove the scratch dir. Follow what br-kf7a did for the install.

## Acceptance

`just check`; a test with concurrent `ensure` calls that all succeed. No migration.

## Model

Haiku if br-kf7a's pattern copies over directly, else Sonnet.
