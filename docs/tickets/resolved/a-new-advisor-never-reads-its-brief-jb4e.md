---
id: jb4e
title: A new advisor never reads its brief
kind: bug
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-6d49]
closed: 2026-10-02T00:43:36.303638Z
---

## The ask


`bridle advisor start <name> --brief ...` sends the brief to `external:advisor` as
"For advisor <name>: ..." and starts `bridle session advisor <name>`. The orchestrator role
says "The advisor picks up its brief at startup", but nothing tells it to. `ADVISOR_PROMPT`
(`crates/bridle/src/session.rs`) says only to prime, check status and the human's open
questions, and say hello, and `bridle prime advisor` never mentions a brief or the advisor's
name. So the new advisor greets the human with a general status report and doesn't know why it
was started.

Seen 2026-10-01 (advisor `workflow`, brief m-3231): the human noticed. The orchestrator typed
a pointer to the brief into the advisor's pane by hand.

The fix: at startup an advisor with a name reads its unread "For advisor <name>:" messages
(the prompt can carry the name, or prime can look the brief up) and starts from the brief when
there is one. Without a brief, today's greeting is fine.

## Resolution

Resolved by: br-6d49 (7bf90f6), br-0e64 (2d722e8)
