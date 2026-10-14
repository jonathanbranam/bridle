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

## Decided

The human, 2026-10-14 ~1:15 PM ET, verbatim: "yes, unify. remus, vesta, dalek all get a home. orch runs there; machine-level mail also good."

- Every machine with an orchestrator gets one: dalek, vesta, remus.
- Name `<host>-home` (dalek-home, vesta-home, remus-home), the advisor's recommendation, not
  objected to: the host in the name because the project registry is shared across machines; not
  plain `<host>`, which reads the same as the machine in `--project` and `@<host>` principals.
- The machine's orchestrator runs in its home project, so dalek's moves out of bridle.
- Machine-level mail on: `<host>-home@` is "mail this machine", read by the orchestrator.
- Repo: local-only git repo with `[state] push = false` (advisor's default; change if the human
  wants it backed up). No workers or managers configured.

## Work

1. Docs: say the orchestrator runs in its machine's home project, not in bridle (the orchestrator
   role, `bridle prime orchestrator`, docs/context/add-a-machine.md with a "home project" step,
   the PdM trial's "Context for agents", and anything that says `--project bridle send
   external:orchestrator`). Senders then address `--project <host>-home`.
2. Setup on each machine (the human's to-dos, or the orchestrator's with the human): repo,
   `[projects]` entry and `bridle projects sync --apply`, daemon as a service, `bridle token pair`,
   `bridle mail install --project <host>-home`, then the orchestrator restarts there. Order:
   remus first (data-contracts stops being its home), then vesta, dalek last (its orchestrator
   is the one everything else addresses today).
