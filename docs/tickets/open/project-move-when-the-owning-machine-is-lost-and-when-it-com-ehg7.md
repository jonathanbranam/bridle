---
id: br-ehg7
title: Project move when the owning machine is lost, and when it comes back (rare; collect the considerations, don't overengineer)
kind: epic
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-kt25, br-53gy]
---

## The ask

The human, 2026-10-12 ~10:40 AM ET (to advisor product-manager, dictated, in the grilling session on kx5u/kt25), verbatim:

> The only situation where that breaks down is where we have lost a computer. Let's have a separate epic for tracking project move with a lost origin, and that's going to be a really rare edge case. I don't want to overengineer it at all. I might not do any of these tickets because, hopefully, that just never happens, and we end up at a state where, in the future, we find different solutions for all these problems. I don't think we need to overengineer anything.
>
> That could be: we should just have a ticket where we're accumulating all of the things, all the considerations that we're aware of right now for what would happen if we lost a machine that owned the project. Those would be:
>
> * We may have lost git pushes.
> * We may have lost bridle state.
> * We've lost any workers or worker agents.
> * We've lost the interactive sessions.
> * Obviously, they don't have a handover node.
> * Obviously, we've lost the old databases.
> * Any outgoing messages will not be delivered, and those could come from any machine within the cluster. We would lose those, or we could reroute those, potentially, but we would have to handle that.
>
> Also note that we need to have a solution in that case for when the lost machine comes back. Let's say my laptop crashes or whatever. I leave it in a hotel for a week and I want to keep working, but a week later I find it, I find Dalek, and turn it back on. For first true reliability, we need to have that button down.
>
> We can just think in terms of cloud servers too. I don't want to pay for cloud servers. That's why I'm using machines that I own, but if you think of a cloud server that's running something that's lost, it could come back in the future. The goal there would be that when that thing comes back online, the rest of the system is resilient to that and realizes that they should tell that system that it's been offline and it's no longer owning the project, and and the other thing that's important is that there needs to be a way. Soon after boot, these demons are going to come up on boot. I don't know what they're going to do exactly, but there needs to be a way to pretty quickly get in there and tell them that the project's been moved

## What this is

A place to collect what we know about moving a project when its owning machine is lost (dead,
stolen, offline for a week), and what happens when that machine comes back. Rare. Not on the path
of epic br-kx5u (the normal move, both machines up). Nothing here is planned or built until the
human decides to; it may never be.

## Considerations so far (the human's list, and the PdM's facts)

When the owner is lost:

1. Git pushes not yet made may be lost (unpushed integration commits, built tickets' branches).
2. Bridle state (`bridle/state`) not yet pushed may be lost.
3. Its workers and their worktrees are gone.
4. Its interactive sessions (orchestrator, aide, advisors) are gone, with no handover notes.
5. Its database (`bridle.db`) is gone: messages, events, usage, schedules since the last push.
6. Outgoing mail to the lost machine's projects, from any machine in the cluster, will not be
   delivered: lost, or rerouted to the new owner (needs handling).

When the lost machine comes back:

7. The rest of the system should be resilient to it: it must learn quickly that it no longer owns
   the project, and someone must be able to tell its daemons soon after boot that the project has
   moved.

## What exists today (facts, 2026-10-12)

- `bridle project take-over --force` takes a project over from a dead machine with no `released`
  mark, "losing whatever it never pushed" (docs/design/cli.md).
- At start, `serve` fetches `origin/bridle/state` and refuses to start when `owner.toml` names
  another machine (docs/design/agent-host/daemon.md, "A project has one serving machine"); under
  launchd or systemd it exits 78 and stays stopped. So a returning machine that can reach GitHub
  already refuses to serve the moved project. Not covered: a machine that boots with no network
  (it reads its local copy), and anything already queued in its outbox.
- The `pre-push` hook refuses an integration-branch push from a clone that is not the owner
  (rule `one-pusher-for-the-integration-branch`).
- kt25's design child K8 (recovery from NEW when OLD is dead: list what may be lost from origin,
  ask, take over, and a returning OLD only unloads its units) moves here from kx5u.

## Related

br-kx5u (the normal move), br-kt25 (its design; K8), br-53gy (stop, take-over, start).
