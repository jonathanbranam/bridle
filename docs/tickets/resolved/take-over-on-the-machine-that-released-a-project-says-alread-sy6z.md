---
id: br-sy6z
title: "take-over on the machine that released a project says 'already owned by this machine': say it is released and how to take it back"
kind: fix
created: 2026-10-15
created_by: external:advisor/wsl2
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-eg64, br-kx5u]
resolved: 2026-10-16T10:02:58Z
---

## The ask

The human, 2026-10-14 (to advisor wsl2), after the data-contracts move: "let's fix the wording".

## What happened (advisor wsl2)

After `bridle project stop` on dalek had released data-contracts (`owner.toml` on origin: `host = "dalek"`, `released = "2026-10-15T00:15:33Z"`), `bridle project take-over` was run on dalek by mistake (meant for remus). It printed:

```
data-contracts: already owned by this machine (dalek)
data-contracts: /Volumes/Data/work/data-contracts-workspace/data-contracts removed from [machine] tools_only; `bridle serve` works here again
```

"already owned" is misleading: the project was released, not owned. Nothing broke. The released mark stayed on origin, remus's take-over then worked, and a `project start` on dalek would have re-claimed it (the daemon's claim clears `released`, `state_branch.rs` ~455-464).

## Why (from the code)

`take_over_project` returns `TakeOverOutcome::AlreadyOwner` whenever `owner.toml`'s host is this machine (`crates/bridle-daemon/src/state_branch.rs` ~1087) without looking at `released`. `crates/bridle/src/take_over.rs` ~62 prints the message.

## What's wanted

Wording only, no change in behaviour. When this machine is the recorded host and the project is released, say so and give the way back, for example:
`data-contracts: released by this machine (dalek) at <time>; it is not owned now. To take it back, run \`bridle project start\` here; to move it, run take-over on the new machine.`
Keep "already owned by this machine" for the unreleased case. The move-a-project runbook's back-out lines may need the same wording.
