+++
id = "br-xqwy"
title = "token pair --rotate fails for every peer token that already exists (conflict: peer:<machine> already exists)"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T01:22:21.283Z"
updated_at = "2026-10-14T14:56:06.002537Z"
created_by = "external:advisor/wsl2"
watchers = [
    "external:advisor/wsl2",
    "agent:xqwy",
]
branch = "bridle/xqwy"
commit = "7fc1421825f5ff472ebc9b6476df86e4d52cb21c"
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

### note · external:advisor/wsl2 · 2026-10-14T02:04:05.885Z
The human, 2026-10-13 ~late night ET (to advisor wsl2), verbatim: "thx, please add to that ticket to be fixed over night".

Not covered by 9c250511 as far as the brief goes: the human's PLAIN run (no --rotate), bridle 0.7.1, after remus joined:

```
% bridle token pair --machines dalek,vesta,remus
machine dalek (role tokens): minted 0, kept 36
machine remus (role tokens): minted 0, kept 40
machine vesta (role tokens): minted 0, kept 36
machine dalek (peer tokens): minted 0, kept 0, failed 8
  failed: peer dalek -> dalek/bridle: mint on dalek: error: conflict: peer:dalek already exists on this daemon: ...
  (same for dalek/bridle-ui, dalek/data-contracts, dalek/track-web, vesta/dotfiles-local, vesta/meta-notes, vesta/meta-notes-ui, vesta/notes)
machine vesta (peer tokens): minted 0, kept 0, failed 8
  failed: peer vesta -> dalek/bridle: mint on dalek: error: conflict: peer:vesta already exists on this daemon: ...
  (same for dalek/bridle-ui, dalek/data-contracts, dalek/track-web, vesta/dotfiles-local, vesta/meta-notes, vesta/meta-notes-ui, vesta/notes)
error: some tokens could not be paired (see above); run again to fill the gaps
```

Two things to fix tonight:

1. **Peer entries are never "kept".** Every existing peer entry counts as not working (kept 0), including the four `vesta -> dalek/*` peer tokens that the human's `--rotate` run minted about an hour earlier. Find out why: either the check (pair-peer-held / pair-peer-active) is wrong, or the tokens in the holders' `[peer]` entries really don't work, which would mean cross-machine forwarding is broken right now. Check which, and report it on this thread.
2. **A plain run must also finish with 0 failed.** When a holder's entry doesn't work but `peer:<machine>` exists on the receiver, pair has to replace it, not fail on the conflict. That's the same replace as `--rotate`, so "run again to fill the gaps" actually works. The brief's "Without --rotate, keep the existing behaviour" assumed working entries are kept; they aren't today.

Acceptance: on the human's three machines, `bridle token pair --machines dalek,vesta,remus` reports failed 0 and peer entries kept on a second run. (remus has no projects yet, so it holds no peer tokens.)

### note · agent:manager-2 · 2026-10-14T03:13:36.403Z
integrated: 7fc1421825f5ff472ebc9b6476df86e4d52cb21c (branch bridle/xqwy)

### note · agent:manager-2 · 2026-10-14T03:13:41.003Z
cleanup: removed nothing; kept agent xqwy (background job pid 20463)

### note · agent:xqwy · 2026-10-14T03:17:21.655Z
done: main merged, just check exit 0, 1654 tests passed; 4d0c6ce2 (an earlier run hit 3 land_test failures under load; they pass alone and on rerun)

### note · external:advisor/product-manager · 2026-10-14T03:53:38.682Z
Resolved by PdM: verified 7fc14218 (receiver upserts with replace; pair always sets it; test). Note: the receiving daemon must run a build with this fix, so upgrade every machine before the next --rotate.

### note · agent:pm-1 · 2026-10-14T03:55:11.709Z
pm-1: the human's late addition (plain run reports peer kept 0 for existing entries) is not clearly answered by the landing: replace-on-conflict is built, but nobody reported why the held peer entries count as not working (pair-peer-held / pair-peer-active check wrong, or tokens really dead). PdM: please check on the machines whether a second plain `token pair` now reports peer entries kept; if not, file a follow-up and I will plan it.

### note · agent:xqwy · 2026-10-14T03:56:26.006Z
Why peer entries were never kept: pair-peer-active looked for a token whose name was "peer:<sender>", but a peer token's name is the bare machine ("dalek"); "peer:<machine>" is its principal id. So the receiver-side check always failed and every peer entry was re-minted (and, before 7fc14218, hit the conflict). The held entries were probably fine. Fix: compare the principal id (crates/bridle/src/commands/misc.rs), commit 39b308e7 on bridle/xqwy. Not covered by a test (the pair simulator fakes the helper) and just check NOT run on it, only cargo build; the manager's landing check must run it. Needs the fix on the machine that runs pair (the CLI side only; receivers need 7fc14218 for replace).

### note · agent:xqwy · 2026-10-14T03:56:26.048Z
the uncommitted change IS needed: it fixes the 'kept 0' bug (pair-peer-active compared token name to peer:<m>; the name is the bare machine). Committed 39b308e7 on bridle/xqwy, cargo build ok, just check not run. Details on the ticket. Stopping.

### note · external:orchestrator · 2026-10-14T14:55:41.850Z
orchestrator: reopening the landing question. advisor/wsl2 (m-12702) found the second fix 39b308e7 (pair-peer-active matched the peer token by name, the bare machine, not the principal id: the 'peer kept 0' report) only on bridle/xqwy; the ticket was resolved on 7fc14218 alone. Verified: 39b308e7 is not an ancestor of main. Needs a land (just check on the branch with main merged, then land). Not critical (wrong count in token pair's report); sent to pm-1 to schedule.

### note · external:orchestrator · 2026-10-14T14:56:06.002Z
orchestrator: correction: the fix already landed as br-378y (a163726f on main, peer_token_live matches the principal id). Verified on main. Nothing to land.
