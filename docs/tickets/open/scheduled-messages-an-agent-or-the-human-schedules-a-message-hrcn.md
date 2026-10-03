---
id: hrcn
title: "Scheduled messages: an agent or the human schedules a message to an agent, once or recurring (cron)"
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [fx7x, cvaq, phyy, cy2v, jttf]
tasks: []
---

## The ask


A feature that needs design before it's built: **no task until the human has refined the design**
(k7tm: a ticket without a task gets no work). The human, verbatim (2026-10-03, via the advisor),
after the orchestrator slept from about midnight to 9 a.m. ET with no wait running (see the
incident of 2026-10-03 in `docs/context/incidents.md`):

> That should never happen. I, I guess I, I don't understand the decision making that went into
> that. Um, the orchestrator serves a critical role in this system, so I understand we have quiet
> hours, and it's fine for you to. So, so other agents can go to sleep completely, but the
> orchestrator should not go to sleep completely. And especially if you can only be woken by me,
> that's a problem. So, you know, given the two-hour Claude timeout limit, no matter what's going
> on, you should wake up every two hours and, you know, do a system check and just, like, check
> your own schedule, right? Like, unless there's some other way for you to wake up at 6 a.m., you
> can't say, I'm going to sleep until 6 a.m. because nobody's going to wake you up at 6 a.m. So
> you need... You need to run those background tasks to be woken up later. So you need to like be
> able to run those in a loop and then check the clock or whatever happens. You know, if we need,
> to, if I don't think we want to get into like cron wake scheduling right now, but that is
> actually we probably do. I've been assuming this wake on. Wake on states thing is, is the end
> all be all solution, but maybe we need a little bit more. I think we still need wake on states,
> and that's fine, or whatever we're calling it now. But why don't you write up a ticket for you
> know research or questions? I forget our names. It's a feature request, but it's not ready to be
> written as a feature until I look through the design. So it's kind of like a design ticket for a
> feature. If that's not a kind or there's not an appropriate kind, let's create that because
> that's something we need to have a lot of. It's a design for a new feature. Maybe that's, maybe
> that's just called a feature ticket, but there's no task. You don't create a task. It needs
> design work first. And that would be the ability for agents... to set a cron for themselves, or
> and for me to set a cron for them as well. And this is going to be something that's critical
> for my work on the notes project because I want that agent to be woken up at a particular
> times. If I want that agent to wake up at 3 a.m., I wanted that that needs to be consistent, and
> so there needs to be a way to. wake that agent up and have it do something. This could be a
> background agent or a, an interactive agent, either one. There should be a way in the system to
> set a schedule. It should be fairly flexible here. So, you know, timestamp is obviously one, and
> then recurrence, you know, every, every hour, and then, what else? Like, I don't know. I think
> cron is probably just the right tool here, unless it lacks something in particular. I think the
> fire once kind of thing is the only thing that maybe doesn't exist in cron. So there, there
> should be like one-time things like wake me up Tuesday at, at 5 p.m. because I need to send a
> message or do something Tuesday at 5 p.m. This is like a personal scheduler kind of thing that,
> that we'll use Bridal to do. Yeah, so I think that's better. Like, I'd much rather have it be,
> like, an agent probably receives a message, right? We don't even really want to schedule a
> wake. We just, I think we determined that messages wake agents. So what it is is really a
> scheduled message sent with contents. I think that's the design. So write that up in a ticket,
> and I'll, I'll work on that later. I'll, I'll refine that design later.

("Bridal" is bridle.)

## The idea so far (the human's)

- **A scheduled message, not a scheduled wake.** Messages already wake agents (rmpq, the
  principal wake), so a schedule just sends a message with set contents at set times.
- **Who sets one:** an agent for itself, and the human for any agent. Background (headless) and
  interactive agents alike.
- **When:** cron expressions for recurrence (every hour; 3 a.m. daily), plus **one-time** ("Tuesday
  at 5 p.m."), which cron lacks.
- **Uses:** the notes project's agent woken at fixed times
  ([[a-life-assistant-agent-on-the-notes-repo-phyy|phyy]]); a personal scheduler run through
  bridle; the orchestrator's periodic system check, so it never depends on the human to wake it.
- Wakes on state changes stay too; this adds time.

## For the design (advisor's notes, not decided)

- Where schedules live (the daemon's database, so they survive restarts) and the CLI
  (`bridle schedule add/list/rm`?), times in the human's zone (US Eastern) with DST.
- What happens to a firing missed while the daemon was down (fire once on start, or skip).
- Delivery to a recipient that isn't running: queued like any message; and with read on delivery
  ([[read-on-delivery-can-lose-messages-marked-read-before-the-re-k8jn|k8jn]]).
- Cross-machine: a schedule lives on one project's daemon, like messages
  ([[one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v|cy2v]]).
- Quiet hours ([[focus-hours-quiet-and-locked-cvaq|cvaq]]): do
  scheduled messages to agents fire during them? (Probably yes; quiet hours limit the human.)
- Related: [[the-orchestrator-stays-running-fx7x|fx7x]] (a watcher for the watchman).

## Kind

The human asked for a kind meaning "a design for a new feature" if none fits. For now this is a
`feature` ticket with no task, which under k7tm means "not ready to build". Whether to add a
separate kind (e.g. `design`) is noted in [[tickets-and-tasks-why-both-k7tm|k7tm]].
