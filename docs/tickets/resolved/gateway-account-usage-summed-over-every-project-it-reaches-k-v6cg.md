---
id: br-v6cg
title: "Gateway: account usage summed over every project it reaches (ktgd step 3; takes over br-qhsa's merged route)"
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
resolved: 2026-10-14T17:22:42Z
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

One gateway route that calls the usage periods and breakdown routes on every project daemon it reaches and sums them, keeping project and machine on each row; unreachable projects listed as unreachable, never zero. Rate-limit readings are account-wide: return them once. Takes over br-qhsa's merged route. Post the final route path and shape on this thread for ui-h79b; add the TS bindings.

Acceptance: a test with two project daemons, one unreachable; `just check`. Docs: human-web-ui.md, api.md. Size M.
