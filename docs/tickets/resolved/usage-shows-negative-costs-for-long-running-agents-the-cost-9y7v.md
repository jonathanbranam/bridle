---
id: br-9y7v
title: "Usage shows negative costs for long-running agents: the cost delta goes wrong when the agent's claude session is replaced"
kind: bug
created: 2026-10-12
created_by: external:advisor/product-manager
theme: performance
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-rdkn]
parent: br-rdkn
resolved: 2026-10-12T23:27:30Z
---

## The ask

Found by advisor product-manager, 2026-10-12 ~6:15 PM ET, checking usage for the human (who asked for "tracing and reporting to know for sure" as the weekly limit nears). The human approved fixing it ("Agree on 1-3. Let's do it").

`bridle usage --by agent --since 7d` on bridle's daemon shows negative costs for the two long-running agents:

```
a-fs6oq (pm-1)       453 turns  110326k tokens  -1.5076
a-qx8ud (manager-2)  666 turns  175054k tokens  -1.3676
```

So `--by role` shows manager -1.37 and product-manager -1.51, and the total ($93.25) is understated.

## Likely cause (to confirm)

`crates/bridle-daemon/src/supervisor.rs` ~1672: a turn's cost is `total_cost_usd - st.last_cumulative`, because claude's `total_cost_usd` is a running total per session. When the agent's claude process is replaced (renew at the context threshold, resume, restart, upgrade) the new session's running total starts again from zero, but `last_cumulative` keeps the old session's value, so the first turn after it records a large negative delta. Both agents renew often.

## The fix

- Reset the running-total baseline whenever the claude session behind the agent changes (or record the per-turn cost directly if stream-json gives one; see docs/spikes/01-stream-json-findings.md). A delta below zero is never stored: treat it as a new session.
- Repair what's stored: a migration or a one-off that corrects the negative rows already in the turns ledger (from the next turn's cumulative, or set to the new session's first cumulative). Say in the summary how many rows changed.
- A test with the fake claude: a renew mid-run, costs stay non-negative and add up.
