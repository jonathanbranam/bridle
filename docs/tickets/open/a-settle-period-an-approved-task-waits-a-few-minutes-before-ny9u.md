---
id: ny9u
title: "A settle period: an approved task waits a few minutes before work starts"
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [refining-a-task-with-the-human-before-it-ships-hvxk]
tasks: [br-3c71]
---

## The ask


The human, verbatim (2026-10-02, via the advisor):

> So it's something else to put it in the backlog to start a ticket to design is that tickets are
> going from like me deciding on the ticket to you to being authored to being implemented like a
> little bit too quickly. And I like the automation of the system that if I, because sometimes I
> walk away from my phone and I'm just completely gone or I walk away from the laptop and I can't
> then reply to something. And so if I say a ticket's approved, it is definitely fine for that
> ticket to be built. But what I'd like is a little bit more time built in to every ticket so that
> when it's filed, there's at least like five minutes, maybe something like that. We can make it
> configurable, but this is part. This should be part of a workflow definition. Something like
> five minutes before that ticket would actually be worked or or task. So you know, I think we
> could. I have to think about what the workflow is today, but I think we, for the advisor and
> orchestrator. You create a ticket, ask me about it, I approve it. We could create the task and
> mark it as ready, I think, whatever the mark is. And I think there's a comment on the task when
> that changes, and that should have a timestamp. And then there should be a mandatory like quiet
> period of about five minutes before that task is actually planned, I think, by the product
> manager is probably where this, this fits. That way, often I find myself coming back in a couple
> minutes or after I said approved, what I check in the output isn't exactly what I meant. And,
> and then the ticket's already half implemented in, you know, like 30 seconds, which is amazing.
> I mean, I love the speed here, but I want just kind of a, a little bit of a breathing room for
> the human to think and process. So I don't want an approval gate. Like I don't want tickets
> piling up waiting for me again. To approve them again, if I've already approved them, or tasks,
> I mean. But I want them, I want just some sort of pause so that if I have a second thought, we
> can, we can go update that and get a fix in before it's actually built.

## What the human wants

- After the human approves a ticket, its task waits a short **settle period** (about 5 minutes,
  configurable) before anyone starts building it.
- **Not an approval gate.** Nothing waits for a second approval; once the period passes, the task
  goes ahead on its own, so work still moves while the human is away.
- The point is room for second thoughts: the human rereads what was filed, and a correction lands
  before the build starts.
- It's part of the workflow definition, configurable (per project), not a role-prompt habit.
- The human's sketch: the approval marks the task ready, the task's thread records that with a
  timestamp, and the settle period runs from it before the product manager plans the task.

## Today (advisor, 2026-10-02)

Today an approved ticket's task can be planned, claimed and building within a minute. 3xr4 was
filed, scheduled and landed the same evening. Nothing in the task lifecycle (`open` → `planned` →
`claimed`, [[docs/design/storage|storage]]) holds a task for a period of time.

## Advisor's notes for the design (not decided)

- **Enforce it in the daemon, not in prompts:** a task isn't startable (`bridle task ready`, the
  queue, `claim`) until the settle period has passed since it was approved. That way the PM, the
  orchestrator on small projects with no PM, and a manager all honour it with no role changes.
- **Restart the clock** when the human comments on or edits the task during the period, so a
  correction gets its own breathing room.
- **Show it:** `bridle task show`/`queue` say "settling until 10:42 AM".
- **Overrides:** a per-task skip for urgent fixes (the human only), and `0` in config turns it off.
- Open: what exactly starts the clock (task created, `plan`, or an explicit "approved" entry in
  the thread), and whether it applies to tasks no human approved (agent-filed chores, bugs a
  worker found).
