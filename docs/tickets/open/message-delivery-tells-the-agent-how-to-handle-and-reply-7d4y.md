---
id: 7d4y
title: Message delivery tells the agent how to handle and reply
kind: feature
opened: 2026-10-06
repos: [meta-notes]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


## The ask

The human, 2026-10-05 about 9:56 PM (relayed by the notes advisor, notes
m-0087). The notes advisor told the human a request "hasn't been confirmed"
because it got no reply, though the meta-notes orchestrator had read it in
seconds and filed the work. Read isn't acted on. The human doesn't trust a
role-prompt rule to be followed consistently, so the guidance should arrive
with the messages, at delivery time. Their words:

> when a message is delivered, or several messages are delivered, the daemon
> should give instructions to the agent on how to handle the messages. It
> should include some guidance, like:
> 1. When you've handled a message and done something with it, and an action
>    has been performed, reply summarizing the action that was performed in
>    brief.
> 2. [We should] have a reminder there that any communication about a task
>    should be done with comments on the task and not through messages. It's
>    sufficient to say, 'Comment added to task,' maybe something very brief.
> 3. Some of our messages are not always about tasks, so we should leave some
>    room for other kinds of replies that might be needed.
> 4. [An] informational-only message doesn't need a reply.

## Wanted

Wherever the daemon hands an agent or session messages (`bridle inbox`,
`bridle agent wake`, `bridle orchestrator wait-for-wake`, the startup
delivery), it appends short handling guidance: once you've acted, reply
briefly with what you did and where (ticket or task filed, "comment added to
<task>", done in vX, waiting on the human, won't do because ...); task
discussion stays on the task; other replies as needed; no reply to FYI-only
messages. Not a new "acknowledged" state: read stays delivery.
