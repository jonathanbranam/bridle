---
id: jttf
title: "Interactive sessions: context for all, daemon-decided wakes, restart with handover, one tagged pane each"
kind: feature
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [a-life-assistant-agent-on-the-notes-repo-phyy, hold-the-orchestrator-relaunch-8fsx, tag-a-tmux-pane-from-bridle-butk, orchestrator-spins-off-an-advisor-ervd, orchestrator-identity-and-recovery-7d62, orchestrator-watches-its-own-context-c9zm]
tasks: [br-b4ac]
---

## The ask


The human, verbatim (2026-10-01, via the advisor), first the question:

> So what kind of tracking and work do we have for agents, interactive agents besides the
> orchestrator? I'm really trying to send all my questions to advisor agents wherever possible.
> But I think they don't wake on messages and they don't have context tracking. They don't have
> automatic restart or handoff procedures, which is not entirely a problem, but the handoff maybe
> could be something we could consider, but I'm just not sure how much we want to add there, but I
> think we can, it'd be nice if we can at least report on their context usage and restart them
> with or without a handoff.

The advisor proposed four steps (context reporting, wake on messages, restart on request with or
without a handover, automatic crash restart and handover). The human's answer, verbatim ("Bridal"
is bridle, "pain" is pane):

> Number one is definitely approved. Let's track context. For every interactive agent.
>
> For the second one, I think what we want is a new command in Bridal for called maybe like Agent
> Wake or something. I, I'm not familiar with the redesign of all the Bridal subcommands, but
> something like that. And it would take, I don't know exactly how this is handled right now, but
> um, it would take the name of the agent, its identifier, and then the bridal daemon can be
> programmatically determine if that particular agent needs to wake up. That way we can move the
> handling of this into bridal itself instead of the agents making decisions on when they should
> wake up. So, um, so if we change the rule later, we just rewrite bridal and not worry about the
> agent prompts or anything. So that way, yeah, bridal could wake an agent because a message is
> ready or because, you know, a ticket has a comment added on it that they should know about or a
> ticket changes status or, sorry, a task changes status or just any, any external condition that
> um, in bridal identifies that should wake the agent. Um, it could wake it on, you know, a
> schedule if it needed to. So let's go ahead and uh, I think that's a better design, more
> flexible and reusable design.
>
> For restart on request, that also sounds fine. I think we need to give a more robust solution to
> the tmux pain problem. So let's just go ahead and assume all interactive agents start in a tmux
> pain somewhere. And we may have to figure out how to, you know, not put too many panes in a
> window. Um, we can come back to that particular problem. But then for now, let's just say that
> whenever we use Bridal to start up an agent, it always tags its window, its, I'm sorry, it tags
> its pane with its identifier. The biggest problem here is that we've already run into this a few
> times, that when an agent ends, that tag on the session pane stays around. In some cases, that's
> a good solution, because then we can restart the agent in the right pane. But in other cases, it
> can be a problem, because there may be two panes tagged with the same identifier. So I'm not
> sure how to resolve that, but it's something to think about.
>
> for crash restart. Yeah, again, I think this is something that needs to be a decision. We really
> need like a better management of these interactive sessions. So in some cases, we don't want it
> to restart. And in other cases, maybe we do. So I think we need something a little more nuanced.
> But I'm not sure exactly how to design that. One example of this is we have like a race
> condition when I am working with the orchestrator and let's say I want to shut it down or
> restart it or do something. Maybe I just am having a system problem and need to turn off, exit
> the agent. When I do that, the daemon starts it back up again. That's my only recourse is to
> shut down the daemon to stop it from doing that. I think. I don't know if there's a way to to
> disable that quickly. The other race condition is when I start a new daemon, if there's no
> orchestrator running, it can start looking for one and start one up. Meanwhile, I'm probably
> starting the orchestrator in a different terminal at the same time. And the other race
> condition is if I go ahead and say, oh, I should start the orchestrator first, then the
> orchestrator can't talk to Bridal at all and starts reporting the daemons down and doesn't know
> what to do. So those two things are, are kind of problematic. And I'm not exactly sure how to
> resolve them because I don't know how Bridal determines the difference between, hey, the human
> shut down the orchestrator on purpose or some other agent versus the agent crashed. It's hard to
> tell the difference. And, you know, the orchestrator, or the, yeah, all of those race conditions
> are just complex.
>
> So I think the main thing I'm saying here is it's already hard enough with the orchestrator with
> these race conditions, so let's not add automatic restart yet. Um, let's keep that with the
> orchestrator and think about a good solution in the future. For the advisors, I do think having
> a handoff would be a good choice. We don't have to rush to implement that, but I think it would
> be valuable. I want to be able to work with an advisor for a while and then have it hand off and
> restart. Or even just restart without context would be acceptable as a first pass. The handoff
> doesn't need to be anything too fancy. I think another issue we have immediately is the inbox
> problem with advisors. If I'm running multiple advisors and they all share an inbox, it might be
> kind of weird. But then again, if I turn the advisor off, then nobody's ever going to read the
> message. So I feel like I'm a little bit stuck there in the best solution. If I have temporary
> advisors, should they be sending messages around? I think they can send messages out, it's fine,
> but the receiving of messages is kind of an interesting problem.

## Today (advisor, checked 2026-10-01)

- **Context:** the global statusline (`bridle statusline`, `~/.claude/settings.json`) already
  writes every session's tokens to `$BRIDLE_HOME/context/<session id>`, advisors included. Only
  the orchestrator's session id is known to the daemon, so only it gets thresholds and
  `orchestrator.context` events ([[docs/design/agent-host/orchestrator-supervision|supervision]]).
- **Wakes:** `bridle orchestrator wait-for-wake` serves `external:orchestrator` only; the daemon
  already decides its wake reasons. `--mail` returns only on email-bridge mail; `bridle wait`
  needs a task. Advisors see messages only when the human next types to them (phyy gap 3).
- **Session files:** `orchestrator.pid`/`.session`/`.exits` are one per machine (phyy gap 2,
  7d62). Only the unnamed advisor writes a pid file (`advisor-<project>.pid`), for mail.
- **Panes:** `bridle pane tag <name>` (butk, built) moves a tag: tagging a pane clears it from any
  other pane, so tags set through bridle are unique. Panes tagged by hand, or a stale tag left
  after a session ends, aren't cleaned up.
- **Relaunch race:** the human exiting the orchestrator gets it relaunched; 8fsx
  (`bridle orchestrator hold`/`release`, br-96a6, open) is the fix for that one.

## Decided by the human

1. **Context tracking for every interactive session** (orchestrator, advisors, any external
   session): bridle knows each session's id and reports its context (events, `bridle status`).
2. **Wakes are decided by the daemon, for any agent.** One command (name TBD, e.g. `bridle agent
   wake <identifier>`) that an interactive session runs and that returns when the daemon decides
   that agent should wake: a message for it, a comment on or state change of a task it should know
   about, a schedule, or any other condition. The rules live in bridle, not in role prompts, so
   changing them is a bridle change. Generalizes `wait-for-wake` (phyy gap 3).
3. **Restart on request, with a handover or without.** Restart with no context is an acceptable
   first pass; the handover needn't be fancy. Not urgent.
4. **Every interactive session bridle starts runs in a tmux pane tagged with its identifier.**
   Too many panes in one window is a later problem. Stale tags left after a session ends are
   useful (restart in the same pane) but can leave two panes with one identifier; to be designed.
5. **No automatic crash restart or handover for anything but the orchestrator.** The
   orchestrator's races stay with it for a future design: the human exits it and it's relaunched
   (8fsx); a new daemon launches one while the human starts one in another terminal; an
   orchestrator started before its daemon can't reach it and reports the daemon down. The daemon
   can't tell a deliberate exit from a crash.

## The advisors' inbox: decided (the human, 2026-10-01)

Advisors share one principal (`external:advisor`) and so one inbox. With several advisors it's
unclear which should read a message; with none running, nobody reads it. The advisor's design,
which the human approved: "Yes, definitely. I approve the design. I think that looks really good."

- **Address:** `external:advisor/<name>`, e.g. `external:advisor/research`, or
  `external:advisor/research@nuc` from another machine (the visitor suffix is unchanged). `/` can't
  appear in a principal name today, so the form is unambiguous: what follows `/` names one session
  of the principal before it.
- **How bridle knows it:** `bridle session advisor <name>` registers the session (`advisor/<name>`,
  pid, session id, pane) with the daemon at launch, the same session tracking decision 1 needs.
  The daemon watches the pid as it does the orchestrator's and marks the session ended when it's
  gone.
- **No new token:** a named advisor uses the shared advisor token. Its CLI adds the name from
  `BRIDLE_ADVISOR_NAME` (already set by `bridle session`), so it sends as
  `from: external:advisor/<name>` and replies come back to it.
- **Delivery:**

  | Case | What happens |
  |---|---|
  | The session is running | Delivered to its inbox; the wake command (decision 2) wakes it |
  | Ended, or never existed | Delivered to `external:advisor`, marked "originally for advisor/<name>"; the sender is told "<name> isn't running; delivered to advisor" |
  | It ends with unread messages | They move to `external:advisor` with the same mark |
  | The part before `/` isn't an active principal | 404, as today |

- **Attribution, not security:** advisors share a token, so the name is an honest label (like
  today's hand-signed "From advisor (research)"), not proof.

## Context warnings and a ceiling for every interactive session (the human, 2026-10-03)

Asked after a table of dalek's seven interactive sessions (contexts 39k to 190k, memory 225 to
626 MB), verbatim:

> Okay, and remind me: can you restart yourself? I don't know where we ended up with this, but
> for all these interactive remote control sessions, we need to start issuing context limit
> warnings and a handoff-type solution of some kind. In some cases, I want a warning before I
> hand off this force, because as the human, I may decide just to shut it down instead of doing a
> handoff. I do want, probably, an upper ceiling on this, but the upper ceiling is going to be
> higher than with the orchestrator to allow me a little more flexibility.

("hand off this force" is likely "a handoff is forced".)

State on 2026-10-03:

- An advisor can't restart itself: nothing relaunches an advisor, and a session can't restart
  its own process. Today the human exits it and starts a new one (`bridle session advisor`); it
  can write a handover note into the repo first.
- Built: advisor session registration and `session.context` events (supervision doc status line).
  Planned, not built: thresholds, warnings and restarts for advisors.
- This ticket's task br-b4ac was **dropped** in pm-1's k7tm sort (2026-10-03); the human's
  decisions above still stand.
- The orchestrator's thresholds, for comparison: note at 150k, plan a handover at 180k, hand over
  at 200k with a 30-minute deadline, and at 12 h uptime.

Wanted (the human's):

1. **Warnings for every interactive session**, Remote Control ones included, as context grows.
2. **A warning before any forced handover**, to the human, so they can choose to shut the session
   down instead of handing over.
3. **A hard ceiling**, higher than the orchestrator's, for more flexibility.

Advisor's proposal for the numbers (the human to set): warn the session and the human at 200k;
warn again at 300k ("hand over or shut down?"); ceiling at 400k: the session writes a handover and
stops after a deadline, unless the human has said to shut it down. Configurable per role.

### Decided (the human, 2026-10-03)

The human, verbatim:

> Okay, I want to think about this again. I think there are two kinds of situations I run into:
>
> 1. I'm talking to an advisor about a whole lot of things. It's not a single-track conversation,
>    so I'd be real happy with a restart around 150. I would do a restart without a handoff,
>    probably. I just don't need a handoff for a lot of those sessions. That's a lot of what I use
>    this session for. I trust bridle to handle everything else, really. I'm kind of happy with a
>    clean slate, so it should always be the human's choice. The human can always choose a
>    restart without a handoff.
> 2. I'm thinking we should start getting warnings around 150 just so that you and I are aware,
>    because when I'm on my phone or somewhere else, it's not easy to see the context at all. I
>    think we could go over 200 by a little bit, so I'd probably warn at 150 and 200. At 250,
>    probably tell it to hand over or shut down. The ceiling is 300. I don't know. The thing is, I
>    don't actually want to go that high. I think what I really want is the ceiling at 250, but
>    warnings at 150 and 200. At 200, it should say you should go ahead and plan a handoff unless
>    the human overrides that. What we want is the ability for the human to override the handoff
>    up until, let's say, 300. If we go over 300, the handoff is forced and the session restarts.
> 3. Let's warn every 50.

> Yeah, ask again at every 50, and I think that looks good. I think the other thing we had in that
> ticket was auto-restarting advisors after a crash, and I want to table that decision for now. I
> think in the future, we're going to have a slightly different system and per-project rules on
> what gets restarted after a crash, but that's too much complexity for right now.

So, for every interactive session (the orchestrator keeps its own numbers):

- **Restart without a handover is always the human's choice**, at any point (a clean slate).
- **Every 50k from 150k, warn the session and the human** (the human can't see context from the
  phone), and ask again each time:

| Context | What happens |
|---|---|
| 150k | Warn |
| 200k | Warn; plan a handover unless the human overrides |
| 250k | The normal ceiling: hand over or shut down, unless the human overrides again |
| 300k | Hard limit: no override; hand over and restart |

- **Crash restart for advisors: tabled.** Later, per-project rules on what restarts after a crash
  (decision 5 above stands meanwhile: no automatic crash restart except the orchestrator).

