---
id: g6v4
title: How agents send the human status updates versus things to act on (the human's running ideas)
kind: question
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [re79, xxxq, 9nrt, rk7k, n2q9, u6wk, rrqe]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 ~9 PM ET, to the bridle-ui aide; dictated). The human will add more
ideas here over time; the bridle-ui aide appends them, quoted. Item 1 of the same message (specs) is
[[projects-write-and-verify-specs-as-they-build-starting-with-9p3v|9p3v]]; the comment-routing change at the end is [[comments-on-a-ticket-an-agent-asked-the-human-to-approve-go-re79|re79]].

> 2. Can you create a new ticket with some ideas on how I can handle a lot of things that I need to
> be doing? I have a lot of sessions running right now, and every one of those sessions is sending
> me messages on things I need to deal with. Some of them are just giving me status updates. One
> thing the UI needs to handle, as we migrate more to working with background agents and less
> remote control, is that those agents need to have a way to send status updates versus action
> items (items that need to be actioned). We have to-dos right now, but I'm not positive those are
> the right things. If they are the right things, which I think they possibly are, but I'm not sure
> yet, then we need to think about how I can filter those and work through them. Something I would
> need is to defer them. If I have some to-dos that aren't relevant until I get back to my computer,
> I should be able to defer those. They should still exist, but they shouldn't show up at the top of
> my queue. Urgent things should be able to have a priority. Things that are assigned to me should
> have a priority and a recency somehow. Again, these are just ideas. I'm not sure how it all works
> yet. For status updates, it's fine to have a history of status updates so that I can catch up on
> what's happened, but what I don't want is this repeated scrolling: "Hey, I'm still waiting for you
> on this. Hey, I'm still waiting for you on this." As we move away from using no control for
> everything, the agent should ask me to assign this to-do, or whatever we decide the structure of
> that thing is, and give it to me. The agent can check if I've handled it, and they shouldn't be
> messaging me about it. They should be interacting with that to-do item. There's a request, maybe
> for me to review a ticket or give a decision on something. If the agent thinks that's urgent or
> needs my attention and it's not being handled, then what they should do is be able to work
> through the bridle system to bring that to my attention. That could be in the form of alerts when
> we have a bubble app, which we will have, or in the form of marking things unread or raising the
> priority of something. These are just ideas for how we move forward with this system, but they're
> all things that we need to build and consider. I want to start keeping track of these, and I'll
> give you more ideas to add to this ticket. The main theme of this ticket is: how do agents send me
> both status updates and things that I need to action in some way?
>
> As we've talked about here, all those actions need to link out to the thing that I need to act
> on. If it says "review and approve this ticket," then I would link to the ticket.
>
> When I follow that link and review the ticket, I have a few options:
> - I'm going to read the ticket.
> - I could just directly approve it. A ticket that's pending or needs approval should have that
>   ability for me to mark it approved directly instead of through a comment.
> - I can use the comment system to send it back to that agent.
>
>
> Actually, that's an interesting issue there. If an agent's asking me to approve a ticket, my
> comments on that ticket should be sent back to that agent, not to a different agent. Let's make
> sure to file that and keep track of that as a change. Take filter into this.

## Context (2026-10-04)

- Today the human's to-dos are tasks created with `--for-human` (planned, claimed by `human`, one
  inbox message pointing at them); they have a priority (high/normal/low) and an `[at restart]` /
  `[at next reboot]` title convention, but no defer and no "unread". Questions are task questions
  (`bridle task ask`), or messages to `human`, which can't be retracted and which the gateway
  doesn't show.
- Watchers and wakes on task changes: [[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]],
  and what watching means for the human: [[what-it-means-for-the-human-to-watch-a-task-9nrt|9nrt]].
- A phone app that prompts the human: [[a-phone-app-that-prompts-the-human-noise-reminders-voice-to-rrqe|rrqe]];
  answering from mobile: [[answering-hitl-questions-from-mobile-u6wk|u6wk]].
