---
id: br-g9qm
title: "token pair fails every visitor token on a machine's first project: needs peer entries it makes later, or never (--projects)"
kind: bug
created: 2026-10-14
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-su62]
related: [br-8uxe, br-kf7t, br-xqwy]
resolved: 2026-10-15T20:52:32Z
---

## The ask

The human, 2026-10-14 (to advisor wsl2), during the remus home-project setup (8uxe): "all these bugs must be filed and fixed".

## What happened (advisor wsl2)

`bridle project start` of the new project remus-home on remus (bridle 0.7.2; the machine's first project, so remus held no `[peer]` entries yet) started the daemon, then its `token pair --projects remus-home` failed every visitor token:

```
machine dalek (role tokens): minted 0, kept 0, failed 5
  failed: human -> remus/remus-home: mint on remus: error: bad_request: no peer token for 'bridle': its human runs `bridle token create --peer remus` there and pastes the token under [peer] in /home/jbranam/.bridle/credentials.toml
  (same for orchestrator, advisor, aide, mail)
machine remus (role tokens): minted 4, kept 0
machine vesta (role tokens): minted 0, kept 0, failed 5
  (the same five, same error)
error: remus-home is running, but its tokens were not all re-created (see above; a machine marked unreachable was skipped): fix it and run `bridle token pair --projects remus-home`
```

No peer section was printed.

## Why (from the code)

- Minting a visitor token on remus-home's daemon checks that the daemon can forward to the visitor's home project (`Outbox::check_destination`, `crates/bridle-daemon/src/outbox.rs` ~228): it needs remus's `[peer] bridle` entry, a token dalek's bridle daemon mints as `peer:remus`.
- `token pair` does role tokens first and peer tokens second (`crates/bridle/src/token_pair.rs` ~216-404), so on a machine's first project the role phase needs peer entries the same run only makes afterwards.
- `--projects remus-home` (what `project start` runs) keeps only remus-home in the project list (`token_pair.rs` ~300), so the peer entry remus needs, `[peer] bridle` (receiver dalek/bridle), is out of scope and never made: the suggested rerun fails the same way.
- The error text tells the human to run `bridle token create --peer` by hand, though `token pair` exists to do exactly that.

## What's wanted

On a machine's first project, `bridle project start` (and `token pair --projects P`) ends with failed 0:

- Mint the peer entries the role phase depends on first (or the peer phase before the role phase), including the holder entries P's daemon needs to forward to the visitors' home projects, even when those projects are outside `--projects`.
- When a mint still fails for want of a peer entry, the error names the `token pair` command that makes it, not `token create --peer` by hand.

Workaround meanwhile: run the full `bridle token pair --machines dalek,vesta,remus` (no `--projects`) twice; the first run's peer phase makes remus's `[peer]` entries, the second mints the visitor tokens.
