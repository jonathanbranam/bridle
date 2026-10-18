---
id: br-7ytg
title: "The machine's home project is the orchestrator's only project: one orchestrator, there, and nowhere else"
kind: feature
theme: orchestrator-per-machine
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-8uxe, br-tj7q, br-ma8e, br-kuw2, br-dg5g, br-uysm, br-8rdn]
---

## The ask

The human, 2026-10-18 ~8 AM ET (to the aide, from the Stages MVP advisor tj7q's "primer question"), verbatim:

> Let's mint the home project idea as the solution for now. I think it has benefits. A few things
> to round out and finalize this approach. Suggest more as needed:
>
> 1. Enforce one orch only on the machine home project
> 2. Default to machine-home, allow override in global config
> 3. Refuse an orch on any other projects
> 4. Refuse messages to orch on any other project

So the home project (br-8uxe, built as a stop-gap) is now THE design for where a machine's one
orchestrator lives, until the machine daemon (br-kuw2) replaces it, if ever.

## Aide's suggested additions (for the human to accept or drop)

- **A. `[orchestrator]` config only in the home project.** A daemon that isn't the machine's home
  ignores `[orchestrator]` and logs/warns at start (or refuses to start). On 10-17 bridle's leftover
  `enabled = true` kept relaunching the orchestrator into bridle and blocked the move (br-8uxe).
- **B. What "refuse messages" (item 4) means for forwarding.** Today a message to
  `external:orchestrator` on another same-machine project is accepted and forwarded to the home
  (br-dg5g). Options: (a) refuse with an error naming the home project (`--project <host>-home`),
  so senders fix their address; (b) keep accepting and always forward to the home, never store it
  locally. Aide recommends (a) for agents (one clear address) with the error text giving the exact
  command, and checking cross-machine mail (outbox) still routes to the target machine's home.
- **C. Every launcher uses the home.** `bridle session orchestrator`, `bridle up`, the supervisor's
  relaunch and `bridle prime orchestrator` resolve the project from the home (item 2's override
  included) and refuse `--project <not-home>`. Roles and docs say so: CLAUDE.md, orchestrator.md, and the advisor primer's "Messaging the orchestrator" (`workflow/base/roles/advisor.md`), which still offers `--project bridle` "until the machine has its home" (advisor tj7q's primer question, m-16617: its 5:15 PM ET note to dalek-home on 10-17 reached the orchestrator at 9:20 PM ET).
- **D. A machine with no home project.** `bridle up` / `session orchestrator` refuse with how to
  create one (or a `bridle machine init-home`), instead of falling back to the cwd's project
  (see br-8rdn for the token side).
- **E. One check.** `bridle doctor` (or `status`) reports the machine's home, the override if any,
  any non-home project with `[orchestrator]`, and where the orchestrator's waiter actually is.

## Open for the human

Accept/drop A to E; for B, pick (a) or (b). Then this can be planned.
