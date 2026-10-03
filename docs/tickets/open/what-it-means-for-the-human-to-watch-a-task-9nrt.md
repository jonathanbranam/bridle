---
id: 9nrt
title: What it means for the human to watch a task
kind: question
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [essy, xxxq]
tasks: [br-66f5]
---

## The ask


Split from [[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]] (task
watchers). For later; not to build now. The human, verbatim (2026-10-03, via the advisor):

> As for the human, I don't follow exactly what you said about the UI. Where, where would that
> live in our system? I think probably I wouldn't subscribe to anything. I don't know. I rarely
> want to be notified of something. So if I did watch a task, I think I would expect it to be a
> message. The other, I mean, the other option is that I think, I think what you're proposing for
> the human, which makes sense is like, yeah, like an unread kind of thing. Like, I don't know
> where that would live. That'd have to live somewhere else, like a new table or something. But,
> you know, yeah, keeping a list of, like, these things have changed. If that's what you're
> proposing, then I guess I understand that. It'd be a new table. You know, these tasks that you're
> watching have changed since you last looked at them. And then if I read a task, I don't know. If
> I call bridal show on a task, that should... expire the red, I should mark it red. If I view it
> in the UI, that should mark it as red. That seems perfectly reasonable. So I'm not sure it's
> necessary at all. So why don't we like not do that yet? And, just write down, file a ticket
> that, you know, something for later future consideration. What does it mean if a human watches
> a task? And we'll come back to it.

("red" is "read"; "bridal show" is `bridle task show`.)

## The two options on the table

- **A message per change** to the human: what the human would expect if they watched a task.
  They rarely want notifications.
- **"Changed since you last looked"**: the daemon keeps, per human and watched task, when they
  last read it (a new table); `bridle task show` or opening it in the UI marks it read; the UI
  (essy) lists watched tasks with unread changes. No messages.

Not decided; the human isn't sure it's needed at all. The human is not added as a watcher by
xxxq until this is settled (tasks the human creates still get `created_by: human`).
