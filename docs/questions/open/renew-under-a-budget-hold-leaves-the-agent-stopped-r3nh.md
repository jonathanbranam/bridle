---
id: r3nh
title: Renew under a budget hold leaves the agent stopped
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## What happened

2026-09-28 17:43 UTC, with the budget governor holding (five_hour past the workday
`hold_at = 85`): the orchestrator ran `bridle renew pm-1` on the idle product manager
(146K context). The CLI printed:

```
error: conflict: budget governor is holding (five_hour, resets 2026-09-28T19:20:00.203+00:00); pass --ignore-budget once that's built, or wait
```

but the old session had already been stopped (events, agent a-fs6oq):

```
23767 agent.stop_requested {"now":false}
23768 agent.state {"from":"idle","to":"stopping"}
23769 agent.state {"from":"stopping","to":"stopped"}
23770 agent.exited {"code":0,"reason":"stdin_closed","signal":null}
```

So a refused renewal leaves the agent `stopped`, not idle in its old session.

## Notes

- The hold check should come before the stop, so a refused renew changes nothing. Or,
  since a renew replaces a session rather than adding load, a renew might not need to
  wait for a hold at all.
- The automatic context renewal (htp6) may take the same path; worth checking.
