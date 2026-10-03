---
id: xxxq
title: "Task watchers: a creator field, a watchers list, and wakes that say what changed and are never lost"
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [jttf, aqtg, 9nrt]
tasks: [br-519b]
---

## The ask


The human, verbatim (2026-10-03, via the advisor), after the advisor said `bridle agent wake`
wakes a principal on changes to tasks it created or claimed, and that the orchestrator's
`wait-for-wake` doesn't:

> Yeah, tell me about the orchestrator getting notified about a task changing state that it
> created. Do we keep the creator of a task? Um, that sounds extremely useful and something we
> should prioritize.

> I'm not quite following all of that. You said we follow who created the task with a text, not
> actually a flag. And then that's interesting. I'm wondering if we just need a like a watcher
> list or a follow list or you know, whatever we call that, suggest a few names, but, you know, it
> just seems like, it should just be something on a task. Like if you want to get notified, there
> should be a list of agents or, or the human or whatever that gets notified when a task changes.
> I mean, it gets like a message or awake, awake or a message. I'm not sure which, what would be
> better.

> I think we're heading towards a good solution, but I'm not sure we've found the best one yet.
> Definitely in agreement about the watchers list. I think that makes a ton of sense. I don't know
> if the creator should always be a watcher. I think we should just add the creator. First of
> all, I think we should have the creator as a field. Okay, so that's something else to do. Well,
> I want to know who creates a task, not by looking at the event log. That seems pretty obvious.
> And then let's have a watchers list. And then when you create a task, you're added to the
> watchers list because you created the task. But then you can remove yourself. If you don't want
> any more messages, you don't care, you can always remove yourself. As to what is sent and what
> happens, I don't know. About the difference between a wake and a message, the wakes, the wakes
> include a message or include details about the wake. So that's, that's pretty cool. That might
> be enough. I don't know. I feel like, is that stuff all deterministically always provided? What
> if the wake isn't running? Like what happens in the time between a wake expiring for some other
> reason and then something else happening before the wake starts again, before the, the waiter
> starts again? Would a wake sent during that time be lost? Or does it queue and then fire
> immediately? I guess that'd be fine. It seems like we have two different systems for sending
> messages. We have wakes and messages. I don't know. I'm not sure about that one.

> Oh, and then the other thing was, should the message, whether it's a wake or a message, let's
> keep talking about that. I don't feel confident yet about what, which one is right. But whether
> it's a wake or a message, um, does it include what changed or any indication of what changed?
> That, I don't know. It seems like it would be, it would save us a bunch of time. And processing
> to just say, you know, task task that has changed from, you know, planning to ready or something
> like that. Or, uh, you know, a new comment was added, you know, a new comment was added by the
> human. At least, like, give some indication of what changed. That's just going to save a lot of
> tokens and attention instead of... telling the agent something changed and then they got to
> read the whole task through and try to figure out what just changed and go parse the event log.
> Like, how often are you parsing the event log? That sounds like a goddamn waste of time.

(The human-watching part is split out to
[[what-it-means-for-the-human-to-watch-a-task-9nrt|9nrt]].)

## Today (advisor, checked 2026-10-03 in `crates/bridle-daemon/src/principal_wake.rs`)

- **No creator field.** `Task` has `claimed_by` but no `created_by`; the creator is only the
  actor of the `task.created` event. To find "tasks I created", every wake check reads up to
  1,000,000 `task.created` events.
- **The rule is "created or claimed"**, hard-coded; nobody can opt in or out.
- **Task changes between waits are lost (a bug).** `bridle agent wake` takes its event cursor
  from the newest event *when the call starts*. A task change that lands while no wait is running
  (between one wake returning and the next call, or across a daemon restart) is never reported.
  Messages aren't lost (an unread message wakes at once), and the orchestrator's `wait-for-wake`
  keeps a persistent cursor, so its wakes queue. Only the task wakes have the gap.
- **A task wake says too little:** `task: br-c4f4 task.note_added`: the task and the event kind,
  not who did it, the old and new state, or the comment. So the woken agent runs `bridle task
  show` and reads the thread to find what changed. (The advisor did that on every br-c4f4 wake on
  2026-10-02; it read the raw event log only for diagnosis, three times that evening.)
- **The orchestrator gets no task wakes at all** until br-2672 moves it onto `agent wake`.

## What's decided (the human)

1. **`created_by` is a field on the task**, set at creation, shown by `bridle task show` and the
   API. Backfill existing tasks from their `task.created` events.
2. **A `watchers` list on the task.** The creator is added at creation; anyone can remove
   themselves (`bridle task unwatch <id>`) or add themselves (`bridle task watch <id>`). Watchers
   are notified when the task changes, never about their own changes.

## To do (advisor's proposal, for review)

3. **Say what changed.** Every notification carries the change itself: the task and its title,
   who did it, and the change: `planned -> ready`; "comment by human: <first 200 characters>";
   "question asked by worker-3: <text>"; priority or kind old -> new. One line each, enough to
   act on or ignore without reading the task.
4. **Never lost:** each principal has a persistent cursor (as the orchestrator's wake does), so
   changes while no wait is running are delivered at the next wait, at once, and a daemon
   restart loses nothing. Fix this first; it's a bug today.
5. **Claimers:** whether claiming a task adds you as a watcher (likely yes, removable), or
   claimers keep today's implicit rule.

## Open: a wake or a message? (advisor's view, not decided)

Bridle has two systems today:

- **Messages** are stored, addressed to one principal, and have read state: the inbox. Good for
  something a person or role says to another. Each one stays until read, so per-change messages
  pile up (one busy task, ten messages).
- **Wakes** aren't stored; the daemon works them out when a wait asks (unread messages, events
  past a cursor). With a persistent cursor (4) and the details (3) they're as reliable as
  messages, and don't pile up: ten changes are one wake listing ten lines.

Advisor's recommendation: **task changes are wakes with details**, and messages stay for things
someone says. Then a wake is "here's everything that changed since you last looked", and nothing
needs marking read. Open for the human; the alternative is a `task_update` message kind that is
marked read on delivery.

## Ready to schedule (the human, 2026-10-03)

The human, verbatim (via the advisor): "I think the design of the watchers solution is fine and
is ready to be filed. The question of wakes and messages, I want to talk about even more."

So 1 to 5 are approved, delivered as wakes (today's mechanism) for now. The wake-or-message
question stays open and may change how notifications are delivered later, not what they carry.

### Wake or message: a correction (advisor, 2026-10-03)

Wakes exist only for interactive sessions (the orchestrator, advisors): the daemon can't write
into a Claude Code session the human started, so the session runs a background wait. Headless
agents (workers, managers, the PM) are processes the daemon owns, and everything reaches them as
a message written to their stdin. So a watcher notification for a headless agent has to be a
message anyway. That points the other way from the recommendation above: **notifications are
messages** (one coalesced line per change, a kind of their own, marked read on delivery), the
message's read state is the bookmark (stored per recipient, so nothing is lost and named advisors
each get their own), and **a wake is only the doorbell** that tells an interactive session it has
unread messages. Still open for the human.

### Decided: messages (the human, 2026-10-03)

The human, verbatim (via the advisor): "Right, okay, so your recommendation is to use messages. I
feel like we need one thing. So if that one thing is messages, then I agree."

Task-change notifications are **messages** (one line per change, their own kind), read on
delivery (rmpq). A wake is only the doorbell for interactive sessions. With messages, the
persistent bookmark (4) is the recipient's read state, so it needs no separate cursor.
