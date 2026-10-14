---
id: br-xqwy
title: "token pair --rotate fails for every peer token that already exists (conflict: peer:<machine> already exists)"
kind: bug
created: 2026-10-14
created_by: external:advisor/wsl2
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-n63z, br-r2nr, br-eurm, br-5cru]
theme: multi-machine
---

## The ask

The human, 2026-10-13 (to advisor wsl2), verbatim, after the second failed run: "is that bug
filed as a ticket to fix?"

## What happened

During the NUC rename (ticket r2nr), run on vesta with bridle 0.7.1, both times:

```
% bridle token pair --machines dalek,vesta --rotate
machine dalek (role tokens): minted 36, kept 0
machine vesta (role tokens): minted 36, kept 0
machine dalek (peer tokens): minted 0, kept 0, failed 8
  failed: peer dalek -> dalek/bridle: mint on dalek: error: conflict: peer:dalek already exists on this daemon: --peer takes the machine that SENDS to this project (the receiver is this machine); to replace its token, revoke it first with `bridle token revoke peer:dalek`
  ... (the same for dalek/bridle-ui, dalek/data-contracts, dalek/track-web, vesta/dotfiles-local, vesta/meta-notes, vesta/meta-notes-ui, vesta/notes)
machine vesta (peer tokens): minted 4, kept 0, failed 4
  peer: minted on the receiver for vesta -> dalek/bridle   (and bridle-ui, data-contracts, track-web)
  failed: peer vesta -> vesta/dotfiles-local: mint on vesta: error: conflict: peer:vesta already exists on this daemon: ...
  ... (the same for vesta/meta-notes, vesta/meta-notes-ui, vesta/notes)
error: some tokens could not be paired (see above); run again to fill the gaps
```

Every failure is a peer token whose `peer:<sending machine>` principal already exists on the
receiving daemon. Peer tokens minted for the first time (`peer:vesta` on dalek, new after the
rename) worked, and so did all 72 role tokens.

## What's wrong

`--rotate` is documented as "replaces working ones" (docs/design/cli.md, `bridle token pair`).
For role tokens it does. For peer tokens it asks the receiver to mint a principal that already
exists, the receiver refuses, and the run exits non-zero. "run again to fill the gaps" can't help:
every rerun with `--rotate` fails the same way. The only way through today is revoking each
`peer:<machine>` by hand, which is what pair exists to avoid (n63z).

## What's wanted

`bridle token pair --rotate` replaces existing peer tokens the same as role tokens (revoke and
re-mint, or a replace on the receiver), so one run succeeds with nothing failed. A test that
rotates a peer token that already exists.
