---
id: br-eg64
title: "project start after a take-over doesn't re-pair other machines' peer tokens to the moved project: their messages queue as invalid token"
kind: bug
created: 2026-10-15
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: [br-8usa]
related: [br-qwd4, br-kx5u, br-g9qm]
---

## The ask

The human, 2026-10-14 (to advisor wsl2), after moving data-contracts from dalek to remus: "yes file the peer token issue - that is a step it should handle".

## What happened (advisor wsl2)

data-contracts moved dalek -> remus by docs/context/move-a-project.md (stop on dalek, take-over and start on remus; remus on 0.7.3). `bridle project start` on remus printed every check as accepted, but its peer section covered remus only:

```
machine remus (peer tokens): minted 1, kept 7
  peer: minted on the receiver for remus -> remus/remus-home
...
check remus peer -> dalek/bridle: accepted
...
```

There was no peer check for dalek or vesta toward remus/data-contracts. The new daemon on remus starts with a fresh database, so the `[peer] data-contracts` entries in dalek's (and vesta's) `~/.bridle/credentials.toml` no longer work. A message sent from dalek right after the start stayed in the outbox:

```
queued o-0146 for data-contracts -> external:advisor; not delivered yet (unauthorized: invalid token), retrying
```

So every message from another machine's project into the moved project queues and never arrives (orchestrator included), while `start` reports success.

## Why (from the code)

`project start` runs the `token pair` code with `projects: [P]` (`crates/bridle/src/project_start.rs` ~450). `--projects` narrows the holders to machines that have P (`Fleet::peer_entries`, `crates/bridle/src/token_pair.rs` ~296: `senders` comes from `self.up[holder]`), so only the new owner gets its outgoing peer entries made. The other machines' entries pointing at P, which the move made stale, are neither re-minted nor checked.

## What's wanted

- After a take-over, `bridle project start` re-pairs every machine's peer token toward P (every holder whose projects send to P), not only P's own outgoing ones, and its final `token check` covers them, so a stale one shows as refused rather than going unseen.
- Check what the runbook's step 5.3 then needs (it already says to send both ways).

Workaround meanwhile: `bridle token pair --tokens peer` with no `--projects` (all machines are holders), run once from any machine.
