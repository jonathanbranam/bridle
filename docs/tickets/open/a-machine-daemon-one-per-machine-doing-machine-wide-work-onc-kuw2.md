---
id: kuw2
title: "A machine daemon: one per machine, doing machine-wide work once, starting, stopping and moving projects"
kind: feature
opened: 2026-10-04
repos: [bridle]
changes: []
specs: []
needs: []
see: [puaf, 9mxw, xypj, wz42, hw6c, k7mw, 3haz]
tasks: []
---

## The ask

A feature that needs design first (no task). The human, verbatim (2026-10-04 ~11:50 ET, via the
aide), while approving the usage-source fix in
[[usage-readings-go-stale-while-agents-work-and-a-stale-readin-puaf|puaf]]:

> Ideally, this would be one machine, not one per project, so that's an enhancement to consider.
> I don't know how much we have running per project, but I really want to scale down what runs per
> project so that I can keep a dozen projects going and have them be very light.
>
> I'm really unsure about the overall approach, in some ways, of running a lot of this redundant
> work between multiple projects all on the same machine. I think it's a cost that we're paying
> that seems quite unnecessary. I'm wondering, in a lot of ways, if a separate master vital daemon
> for the machine (we could work out what that's called) is always running a master vital for the
> machine. It's not tied to your project, and that can handle machine-level things like this and
> machine-level coordination. That could possibly address a lot of problems with message.
>
> Still agree. Ideally, in my perfect future here, that machine-wide daemon would be able to start
> and stop demons on machines besides project-specific demons, because I want that level of
> coordination. I want to have one place I can say, "Hey, time to do a shutdown of every project
> on this machine, please. Take care of that and indicate: do it safely, wait for work to finish,
> and do it immediately if something needs to reboot."
>
> Another design consideration is that, ideally, that machine-wide daemon would be able to start a
> project up from nothing and do everything:
>
> * clone
> * set up the work tree
> * set up everything
> * initialize the project as it exists
>
> That's one requirement. Another one is that I want that master machine daemon to be able to
> handle transferring projects between machines. If I have three machines running, I should be able
> to say, "Trackweb is running on my laptop. I want to transfer it to the Windows PC." It'll handle
> persisting everything, all agents doing a handoff, doing the `git push`, and then doing the
> takeover command. I'm then communicating with the other machine that's receiving it, saying,
> "Everything's done, ready to receive," and then it will do `git pull` and restart everything that
> was going on on the first machine.

And right after:

> This is a big design, so please open a ticket for it. I want to review it in some detail before
> anything is scheduled. This should address message delivery as well.

("Master vital daemon" is the dictation; the name is still to be chosen.) **Not to be scheduled
until the human has reviewed the design in detail.**

## What it would own (from the ask)

1. **Machine-wide work done once, not once per project.** First case: the usage reading (puaf).
   The goal is per-project daemons light enough to run a dozen projects on one machine.
2. **Machine-level coordination, including message delivery**: mail between daemons on one
   machine and across machines
   ([[daemons-deliver-mail-to-each-other-across-machines-store-and-3haz|3haz]]). The design has to
   say how delivery works with a machine daemon in the path.
3. **Start and stop project daemons**, including "shut down every project on this machine":
   safely (wait for work to finish), or at once when a reboot is needed.
4. **Start a project from nothing**: clone, worktrees, setup, initialise the project as it exists.
5. **Move a project between machines**: the sender persists state, has every agent hand off,
   pushes and runs takeover, then tells the receiver it's ready. The receiver pulls and restarts
   what was running on the sender.

## Related

- [[project-machine-and-account-scope-9mxw|9mxw]]: what's per project, per machine, per account.
  This ticket is one answer to it.
- [[how-project-daemons-share-one-budget-xypj|xypj]]: the shared budget, an obvious machine-daemon job.
- [[a-big-red-button-pause-bridle-on-a-machine-in-levels-by-hand-wz42|wz42]]: pausing a machine
  in levels, which overlaps 3.
- [[one-machine-owns-a-project-hw6c|hw6c]] and [[projects-on-other-machines-by-config-k7mw|k7mw]]:
  which machine owns a project, which 5 would move.
- [[daemons-deliver-mail-to-each-other-across-machines-store-and-3haz|3haz]] and the seats ticket
  (gtzx), both waiting on the human's review, may change shape if this exists. Worth reading
  together before either is planned.
