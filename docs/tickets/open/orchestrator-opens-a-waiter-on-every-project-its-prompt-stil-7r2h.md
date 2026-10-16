---
id: br-7r2h
title: "Orchestrator opens a waiter on every project: its prompt still says to, though one waiter on the home daemon is enough"
kind: fix
created: 2026-10-16
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [[br-kuvh, br-rhba, br-8uxe, br-3haz]]
---

The human, 2026-10-15 ~9:00 PM ET (to advisor product-manager), verbatim:

> Why do some agents, especially the orchestrator, keep holding open a waiter on every project? I thought we fixed this. I don't know. It's like we've been around and around and around this. Most of the agents aren't doing this, but I think for some reason the orchestrator still is.

And, on the diagnosis below:

> Yes, let's file that as a ticket and fix it. That's exactly what I'm seeing.
>
> In some of the other sessions, I think Dalex Orchestrator is writing a bunch of notes to itself and actually solving a bunch of problems because it has a lot of notes. It has some running logs and handoff notes, so it has a better idea of what's going on.
>
> We aren't actually fixing some of the orchestrator prompts. That's what's happening. The orchestrator, I think, on maybe Remus and Vesta, is listening. Opening 4 listeners. Let's get that fixed.

## Why it happens (advisor product-manager, read in the repo 2026-10-15)

`workflow/base/roles/orchestrator.md` contradicts itself:

- Line ~39 ("At every start"): "check that each project you hold a token for has a waiter open (Watch below). Start the missing ones before anything else."
- Line ~85 ("Watch, don't poll by hand"): "keep one waiter open for each project's daemon you hold a token for (listed under `[orchestrator]`)".
- Lines ~99-104: "One waiter watches one daemon: your home daemon ... so the one waiter is enough. A project whose daemon has no visiting `orchestrator@<machine>` token for you still needs its own waiter".

3haz slice 5 (br-rhba, docs/design/agent-host/orchestrator-supervision.md "Wakes are also mail") made one waiter on the home daemon enough, but the startup step and the Watch duty were never rewritten. A fresh orchestrator session reads the startup step first and opens a waiter per project. dalek's orchestrator happens to have learned otherwise through its own notes and handovers; remus's and vesta's, with fewer notes, follow the prompt. The fix belongs in the prompt, not in one orchestrator's notes.

Seen on dalek 2026-10-15 ~8:55 PM ET: one orchestrator waiter (bridle). bridle-ui and track-web had no orchestrator waiter and, with no dalek-home yet, no forwarding either: blind spots.

## The ask

1. **One waiter, on the home daemon.** Rewrite the startup step and the Watch duty in `workflow/base/roles/orchestrator.md` to say so, consistently with lines ~99-104. Remove "each project you hold a token for" as a duty. Check the other roles and `bridle prime orchestrator` output for the same leftover (aide, advisor and any rule naming per-project waiters).
2. **No project left unwatched.** Every project daemon on a machine whose orchestrator lives elsewhere (its `<host>-home`) holds the visitor token `external:orchestrator@<machine>` with that home, so its wakes are forwarded. Where it's missing the orchestrator is told how to set it up (one command), not to open another waiter. Until a machine has its home (dalek today), the prompt says what to do for the projects that aren't the one it runs in.
3. **Detect it.** The orchestrator, at startup, checks (or `bridle status` shows) which of its projects neither forward to its home nor are its home, and names them.
4. Relates to kuvh (remove `--all-projects`), which this unblocks.

Done when: a freshly started orchestrator on remus or vesta, following only its prompt, opens exactly one waiter; every project on that machine still wakes it.
