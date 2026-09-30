---
id: p4ks
title: A worker resumed after a restart dies on its first turn
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [r3nh]
---

## What happened

Worker `python-pack` (a-s3ije, Sonnet, task br-7678) had just handed off for a context
renewal ("Handing off now since context is winding down") when the human restarted the
daemon (2026-09-28 17:53 UTC, on c533cb0). After the restart it showed `idle` with context
`0`. Each `bridle resume python-pack` came up idle, then died on its first turn with no
tokens used (events, agent a-s3ije):

```
23841 turn.ended {"cost_total":0.0,"is_error":true,"n":5,"result":null,"subtype":"error_during_execution",...}
23842 agent.state {"from":"idle","to":"exited"}
23843 agent.exited {"code":1,"reason":"eof","signal":null}
23875 agent.resumed {"from":"exited"}
23880 turn.ended {"cost_total":0.0,"is_error":true,"n":6,"result":null,"subtype":"error_during_execution",...}
23882 agent.exited {"code":1,"reason":"eof","signal":null}
```

The budget governor was holding at the time (five_hour past 85%).

## Notes

- Likely cause, unverified: the agent's stored Claude Code session id points at the session
  it was renewing away from, or at one that never started, so `--resume` fails at once.
  The daemon's stderr (in the human's terminal) would say.
- Its work is safe: committed at 482825b on `bridle/python-pack`, with a handoff note on
  br-7678.

## Findings

- A renew persisted its fresh session id before claude had written that session; a
  restart in that gap left `resume` running `--resume` on a session that doesn't exist.
  That half was fixed earlier (`agents.session_started`). Renew itself does record the new
  session id (`set_agent_session`).
- The remaining case is a session marked started that claude no longer has. It exits at
  once and, as seen here, dies on the first turn.

## Resolution

A `--resume` process that exits abnormally before any turn ended without `is_error` is
treated as a dead session: the daemon marks the session unstarted and resumes once more on
a fresh session (`Session::New`) in the same worktree/branch/role, with the usual
continuation note. `agent.exited` now carries `stderr_tail`, and the exit is logged at
warn. See "Resume" in [agents.md](../../design/agent-host/agents.md). Budget-hold
behaviour is out of scope (br-1392).

Resolved 2026-09-29.
