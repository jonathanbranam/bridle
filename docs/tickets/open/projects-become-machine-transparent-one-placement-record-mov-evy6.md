---
id: evy6
title: "Projects become machine-transparent: one placement record, move on command, later workers on several machines"
kind: feature
opened: 2026-10-10
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The human, 2026-10-10 ~10:25 AM ET (to the aide, relayed verbatim to the PdM), after asking how
br-hdbj works on the NUC:

> What we need is for a project to be tied to a machine, and that's tracked in configuration.
> It's also tracked in the repo. It's tracked in a few places, and we're going to have checkouts
> of projects on three machines pretty soon. The long-term vision here is to have a project set
> up on every machine and for work to be able to be transferred between the machines on command.
> Obviously, that would include shutting down, waiting for work to land, and all of that stuff.
> It should essentially start to become transparent to bridle which machine a project is running
> on.
>
> We're still committed, for now, to one project being only assigned to one machine, but that's
> also something that, in the future, should become more flexible. That one worker: we could have
> workers on different machines. Again, some of that is very future-looking stuff for the product
> manager.

## What exists (the aide, 2026-10-10)

"Which machine owns a project" lives in at least three places: `owner.toml` on `bridle/state`
(moved by `bridle serve --take-over`, and since br-hdbj enforced by a pre-push hook), the machine
and port in `~/.bridle/config.toml` `[projects]`, and the credentials file.

## The vision, as items (future; not scheduled)

1. One source of truth for project-to-machine placement; the other places derive from it.
2. Every project cloned and kept in sync on every machine: ticket rjd5.
3. Move a project on command, with a drain (stop new work, wait for landings, hand over, start on
   the new machine): ticket kt25.
4. Machine-transparent addressing: agents and the human reach a project's agents without knowing
   where it runs (3haz forwarding is a start).
5. Later: one project's workers on more than one machine (kuw2, the machine daemon, is related).

Today's rule stays until the human changes it: one project, one machine.
