---
id: br-8uxe
title: "A home project on each machine for the orchestrator: a stop-gap for the machine daemon"
kind: feature
created: 2026-10-14
created_by: external:advisor/product-manager
theme: orchestrator-per-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kuw2, br-j6d3, br-gb97, br-ma8e, br-jw75, br-afzu]
---

## The ask

The human, 2026-10-14 (to advisor product-manager), verbatim: "I'm considering starting a "machine" project for each machine where we run the machine orchestrator. This will be stop-gap until the final machine-wide solution is written. Each machine currently has one "home" project already, bridle here, meta-notes on vesta, etc. But it doesn't make sense for data-contracts to be the "home" project on remus;"

And: "there are a lot of things that don't exist on a machine until a project lives there", with this list (pasted by the human):

- A daemon, so self-upgrade to new releases actually happens.
- A home for the orchestrator (and later the machine admin), its schedules and its waiters.
- A target for token pair.
- A mail bridge. With afzu every project is enrolled, so remus-home@ would take mail. Harmless, and maybe even useful as "mail this machine".

Then: "yes, file that".

## What's wanted

Each machine running an orchestrator gets a small project of its own, the machine's home: it holds
the orchestrator (and later the machine admin), their schedules, waiters, handovers and incidents,
and brings the machine-level services that only exist where a project lives (daemon and
self-upgrade, token pair, mail). No workers by default, so the daemon stays light. A stop-gap for
the machine daemon (kuw2): when kuw2 is built it takes this project's place.

Open points (for the human):

- Name (see the thread): remus-home, remus-machine, remus-global, or plain remus.
- Dalek too, moving its orchestrator out of the bridle project (the trial context and role docs
  say it runs there), or remus first?
- What the repo is: local-only git repo or one with a remote (state push).
- Mail to the machine project: "mail this machine" or ignored.
