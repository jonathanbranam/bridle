---
id: br-v58q
title: "bridle usage --week and --by name: the account breakdown in the CLI (ktgd step 4)"
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: [br-pemk, br-zn6s, br-bm7a]
related: [br-ktgd]
theme: performance
parent: br-rdkn
resolved: 2026-10-14T11:11:31Z
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

`bridle usage --week` (the week from the reset, with its days and five-hour windows, peaks and hits) and `--by name` (and session), from the daemon routes, so it works without the UI.

Acceptance: CLI test against the fake daemon; `just check`. Docs: cli.md, CHANGELOG. Size S.
