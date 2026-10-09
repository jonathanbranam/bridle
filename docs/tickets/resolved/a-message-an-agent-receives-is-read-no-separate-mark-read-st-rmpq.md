---
id: rmpq
title: "A message an agent receives is read: no separate mark-read step, no unread for agents"
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [xxxq, r8kv, fbfy]
tasks: [br-ee78]
closed: 2026-10-09T23:11:04Z
---

## The ask


The human, verbatim (2026-10-03, via the advisor):

> Right, okay, so your recommendation is to use messages. I feel like we need one thing. So if
> that one thing is messages, then I agree. I have a few questions about messages, though.
> Particularly to agents. I've had some trouble, and I don't know what the source of this is or
> what happened with it, but I had the one of the agents on the NUC sending messages to the
> orchestrator, and the NUC agent kept saying, none of my messages are marked as read, but I'm
> pretty sure that the orchestrator had read them. I mean, I know the orchestrator had read them.
> I just can't tell if it was a race condition or a bug or what was really happening. But, like,
> was the agent reading old information or relying on memory, or did they actually check again to
> see if the messages were read? That part I don't know the answer to. But my question is, you
> know, how does an agent... mark something as red. Is it an action? Do they have to like run a
> command to do that or something? Or is merely the fact that a message was received mark it as
> red? I guess my, my point here is like I want that to be really clear and not confusing. Like, I
> think if a message is consumed by an agent somehow, it should be just marked as read. It
> shouldn't be a separate flow. Like, it's not a human. Like, I don't think, you know, I don't
> know if an agent should be able to leave some, you know, if, if the message goes into the
> agent's context, it should just be marked as read automatically. Is what I think. So tell me if
> that is how it works or if that perception is wrong. Because a human, as a human, I use my inbox
> as a task list, right? Just because I've read something doesn't mean I did anything with it.
> So, I may have forgotten immediately. But the agents don't work like that. They've, you know,
> these, and, and they met, it's the type of messages that are important here, right? Like, some
> messages are just like, like we're, we're creating kind of event, an event delivery system,
> right? So I want to talk about that too, and whether we should be building our own secure
> message delivery system or just like using one that exists or something. But, That's a, that's
> a separate question, but, you know, something to think about. And then, you know, what is a
> message? What does it mean for an agent to have read a message? And, like, should an agent be
> able to mark a message as unread? I don't mind if that's a, if that's a possibility, but what
> does that mean if we're using messages... as an event log, as a, like a, more like an event
> system. If an agent marks it and the message is unread, is it going to immediately be delivered
> again to the agent? You know, so that doesn't make any sense. So I would, I would assume, no, an
> agent cannot mark a message as unread. So if we're good with that, I think, I think we're on the
> same page.

Context: the human agreed task-change notifications are messages, so messages are bridle's one
notification system (xxxq). The delivery-system question is
[[build-our-own-message-and-event-delivery-or-use-an-existing-fbfy|fbfy]].

## Today (advisor, checked 2026-10-03, `docs/design/agent-host/messages.md`)

- **Reading is a separate step for every agent.** States are `pending -> written -> delivered ->
  read`. A headless agent gets the message written into its conversation (`delivered`), but it
  stays unread until the agent runs `bridle inbox --mark-read` ("an agent's inbox also shows
  messages already delivered to it over stdin until it runs `bridle inbox --mark-read`").
- **Interactive sessions** (orchestrator, advisors) get nothing written in; they read with
  `bridle inbox`. Only `--mark-read` or `bridle inbox read <id>` marks read. `bridle inbox`
  without it, `inbox show`, and the orchestrator's `wait-for-wake` (which prints message text and
  doesn't depend on read state) all leave messages unread.
- **Anyone can mark a message unread** (`bridle inbox unread <id>`).
- **The NUC case** is explained by this, likely without a race: the orchestrator can see and act
  on a message (in its wake text or `bridle inbox`) and never mark it read, so the sender sees
  "unread" correctly. Not verified for that case. On dalek, 5 of the 40 messages to the
  orchestrator since 2026-10-02 are still unread.

## Decided (the human)

1. **A message that reaches an agent's context is read, automatically.** No separate mark-read
   step for agents:
   - headless agents: read when delivered into the conversation (the stdin ack);
   - interactive sessions: read when bridle hands it over: the wake returns the messages
     themselves (not just ids) and marks them read; `bridle inbox` run by a non-human principal
     marks what it lists read.
2. **Agents can't mark a message unread.** A message isn't a to-do; an unread one would just be
   delivered again.
3. **The human keeps read and unread** as they are: the human uses the inbox as a task list
   (reading something doesn't mean it's done).

## Notes (advisor)

- Remove `--mark-read` from the advisor and orchestrator role prompts once 1 lands (e.g. the
  advisor loop's "read `bridle inbox --json --mark-read`").
- With 1, a sender's "unread" means truly not yet received, which is what the NUC agent needed.

## Resolution

Resolved 2026-10-09 by advisor (product-manager). Done: the work landed in this ticket's tasks (see `tasks:`). Resolved in the PdM's sweep of tickets whose tasks were all finished.
