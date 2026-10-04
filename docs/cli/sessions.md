# Sessions

Two kinds of Claude Code process run under bridle.

## Hosted agents (headless)

`bridle agent spawn <role> --prompt "..."` makes a worktree and branch, and starts one
`claude -p` stream-json process the daemon supervises. The daemon records its events,
usage and messages.

```
bridle agent list|show <agent>      # state, role, model, branch
bridle agent interrupt <agent>      # stop the current turn
bridle agent stop|resume <agent>    # stop; resume the same session
bridle agent renew <agent>          # fresh process and session, same worktree
bridle agent logs <agent>
bridle agent rm <agent>             # remove the worktree and record
```

An agent that nears its context limit gets a `Context handoff:` note; `renew` starts it
fresh. The budget governor holds new spawns and winds agents down when account usage
is high (`bridle usage`).

## Interactive sessions

`bridle session orchestrator|advisor [name]|aide` starts the role's own `claude` session
from any directory in the project: lean settings, a pane tag, and an opening prompt from
`bridle prime <role>`. The orchestrator's session is relaunched by the daemon after
`bridle orchestrator handover done`. Interactive agents wait for messages with
`bridle agent wake` / `bridle orchestrator wait-for-wake`, which return unread messages
and mark them read.

## More

`docs/design/agent-host/agents.md`, `docs/design/agent-host/orchestrator-supervision.md`,
`docs/design/usage-and-budget.md`.
