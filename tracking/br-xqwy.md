+++
id = "br-xqwy"
title = "token pair --rotate fails for every peer token that already exists (conflict: peer:<machine> already exists)"
kind = "bug"
state = "built"
created_at = "2026-10-14T01:22:21.283Z"
updated_at = "2026-10-14T01:57:27.831340Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:xqwy",
]
branch = "bridle/xqwy"
commit = "9c250511"
summary = "PeerTokenCreateRequest gains replace (serde default false); the receiver's create_peer_token upserts when set, so the old peer token stops as the new one exists. pair-peer-mint (used only when minting is needed) always sets it, so --rotate no longer hits 'already exists'; non-rotate keeps existing peer tokens as before; manual 'token peer' still refuses a duplicate. Test: store replacing_a_peer_token_swaps_it_in_one_step. Docs: cli.md, CHANGELOG. Caveat: if the holder's store fails after mint, the old token is already dead (same as role-token rotate); rerun fixes it."
ticket = "xqwy"
+++

docs/tickets/open/token-pair-rotate-fails-for-every-peer-token-that-already-ex-xqwy.md

## Thread

### note · external:advisor/product-manager · 2026-10-14T01:25:16.174Z
The human, 2026-10-13 ~9:25 PM ET (to advisor product-manager), verbatim: "a usage bug / minor issue in token pair that should be fixed tonight; i'm working through the nuc rename right now and then project transfer and remus set up." Readied by human via advisor (product-manager); fix tonight.

### note · agent:pm-1 · 2026-10-14T01:25:29.239Z
Brief (pm-1). Model: Sonnet. Size: S. TONIGHT, top tier: the human is mid NUC rename / project transfer / remus setup. The ticket's "What's wanted" is the spec. Start: crates/bridle/src/token_pair.rs (the peer-token branch of `--rotate`; the role-token branch already replaces) and the receiver's mint-peer handler in bridle-daemon (the "conflict: peer:<machine> already exists" error; `bridle token revoke peer:<machine>`). Fix on the receiver or in pair, whichever is smaller and keeps the safety (the old token must stop working once the new one is minted and delivered; if delivery fails, do not leave the pair with neither): revoke-then-mint, or a replace-on-mint that is atomic. Without `--rotate`, keep the existing behaviour (existing peer tokens are kept, not failed). Test: rotating a peer token that already exists succeeds, the old token is rejected, the new one works; one run reports nothing failed. Docs: docs/design/cli.md (`token pair`), CHANGELOG.md. Migration: none. Out of scope: the PATH problem (br-5cru), one-token-per-machine.

### note · agent:xqwy · 2026-10-14T01:57:27.699Z
built: 9c250511 (branch bridle/xqwy)

### note · agent:xqwy · 2026-10-14T01:57:27.831Z
done: token pair --rotate replaces existing peer tokens; just check exit 0, 1651 tests passed; 9c250511
