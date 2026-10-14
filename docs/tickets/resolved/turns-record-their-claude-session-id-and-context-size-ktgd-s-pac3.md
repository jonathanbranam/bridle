---
id: br-pac3
title: Turns record their Claude session id and context size (ktgd step 1)
kind: feature
created: 2026-10-13
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ktgd]
theme: performance
parent: br-rdkn
resolved: 2026-10-14T02:22:54Z
---

Split from br-ktgd (held; it holds the human's words and decisions 1-7, read them first). Epic br-rdkn. The human, 2026-10-13 ~7:45 PM ET: "split up into nice little chunks."

## The ask

Add `session_id` and `context_tokens` to `turns` (one migration; old rows NULL, never zero), filled from what each turn already reports (never wake an agent). Then a hosted agent's history splits into its sessions (ktgd decision 6) and each turn's context is kept (decision 5, startup cost).

Acceptance: an agent renewed once has turns under two session ids; context per turn stored; `just check`. Docs: storage.md, CHANGELOG. Size S.
