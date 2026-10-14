---
id: br-378y
title: token pair still reports 'kept 0' for live peer tokens (follow-up to br-xqwy)
kind: bug
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

After br-xqwy landed, 'bridle token pair' still reports 'kept 0' for existing peer tokens: PairPeerActive (crates/bridle/src/commands/misc.rs) compares the token's name to 'peer:<machine>', but a peer token's name is the bare machine; the principal is peer:<machine>. Fix is on bridle/xqwy as 39b308e7 (compare t.principal). Run just check, add a test if one fits, land. The human is using token pair tonight; covered by their go on br-xqwy.
