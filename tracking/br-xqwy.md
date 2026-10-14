+++
id = "br-xqwy"
title = "token pair --rotate fails for every peer token that already exists (conflict: peer:<machine> already exists)"
kind = "bug"
state = "claimed"
created_at = "2026-10-14T01:22:21.283Z"
updated_at = "2026-10-14T01:28:00.809143Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:xqwy",
]
ticket = "xqwy"
+++

docs/tickets/open/token-pair-rotate-fails-for-every-peer-token-that-already-ex-xqwy.md

## Thread

### note · external:advisor/product-manager · 2026-10-14T01:25:16.174Z
The human, 2026-10-13 ~9:25 PM ET (to advisor product-manager), verbatim: "a usage bug / minor issue in token pair that should be fixed tonight; i'm working through the nuc rename right now and then project transfer and remus set up." Readied by human via advisor (product-manager); fix tonight.

### note · agent:pm-1 · 2026-10-14T01:25:29.239Z
Brief (pm-1). Model: Sonnet. Size: S. TONIGHT, top tier: the human is mid NUC rename / project transfer / remus setup. The ticket's "What's wanted" is the spec. Start: crates/bridle/src/token_pair.rs (the peer-token branch of `--rotate`; the role-token branch already replaces) and the receiver's mint-peer handler in bridle-daemon (the "conflict: peer:<machine> already exists" error; `bridle token revoke peer:<machine>`). Fix on the receiver or in pair, whichever is smaller and keeps the safety (the old token must stop working once the new one is minted and delivered; if delivery fails, do not leave the pair with neither): revoke-then-mint, or a replace-on-mint that is atomic. Without `--rotate`, keep the existing behaviour (existing peer tokens are kept, not failed). Test: rotating a peer token that already exists succeeds, the old token is rejected, the new one works; one run reports nothing failed. Docs: docs/design/cli.md (`token pair`), CHANGELOG.md. Migration: none. Out of scope: the PATH problem (br-5cru), one-token-per-machine.
