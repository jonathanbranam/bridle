---
id: br-8umh
title: A failed push is an event and an alarm; an agent that can't send puts the blocker on the task
kind: feature
created: 2026-10-09
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-j7r4, br-2ax5, br-2mtr, br-8ay6]
resolved: 2026-10-10T23:27:37Z
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 4. The human, 2026-10-09 14:04 EDT,
verbatim (comment c4 on j7r4): "Agreed."

The ask:
1. The merger's push after a landing is a bridle command (or `bridle task land --push`) that
   records a `push.failed` event and notifies the orchestrator and the human when the push is
   rejected. A failed push never depends on the pushing agent's own session text.
2. An agent that can't send a message puts the blocker on the task thread (`bridle task
   comment`) instead; add that to the worker and manager roles. (In the incident, manager-2's
   `bridle send` failed and its report was lost; that send failure is incident 2ax5.)

Related: 8ay6, 2mtr.
