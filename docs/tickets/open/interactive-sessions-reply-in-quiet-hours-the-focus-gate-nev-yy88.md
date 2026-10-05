---
id: yy88
title: "Interactive sessions reply in quiet hours: the focus gate never reaches background wakes"
kind: incident
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq, v3b7, 9s8u, mvtz]
tasks: [br-yy88]
---

## The ask

The human, verbatim (2026-10-04 ~10:30 PM ET, to the bridle advisor, during quiet hours):

> I'm not sure when your last message was sent, but you should be receiving quiet time
> notifications and not saying, "I'm waiting for your message." Are you getting that in your
> prompt, or is this a session I haven't properly restarted yet? When did this session start, and
> what's your context right now? Also, are you getting quiet mode notifications?

and, after the advisor's answer (the gate reached it only with the human's own message):

> That's fascinating and somewhat of a problem, so let's file a ticket about that. We need to
> remember this.
>
> I'm thinking about whether it's a post-mortem or a new rule for the bridle while we're developing
> it. Agents that interact with agents that send messages to the human should have, for one, quiet
> mode inserted, but there are probably other hooks they should have inserted. We need to think
> about, every time we're adding a hook, whether that hook also should be included as part of
> background messages.
>
> I would like a post-mortem of what just happened written up, or maybe both. Let's just file it as
> an incident and a post-mortem. Tokens are cheap, and then file a ticket to get that fixed.
>
> I'm not sure how the system works exactly, but when an agent that interacts with the human gets a
> message, a background message, or awake or something, any sort of notifications they would
> normally get need to come with that. I don't know what else it would include right now, but
> definitely that.

## What happened

Quiet hours (`[[focus]]` "weekday-sleep", 21:30 to 00:00 ET, then "everyday-sleep" to 06:00) began
at 21:30 ET on Sunday 2026-10-04. After that the bridle advisor replied to background wakes as in
the daytime: several sentences, links, "I'm waiting for your message". It did so at about 21:56 (a
daemon restart had ended its wait) and 22:28 (the orchestrator's ticket-link instruction). The
quiet-hours limits reached it only at about 22:35, on the human's own message.

## Cause

The focus gate is a `UserPromptSubmit` hook (`bridle focus gate`, `FOCUS_GATE` in
`crates/bridle/src/session.rs`). It fires only when the human submits a prompt. A background
command finishing (the session's `bridle agent wake` waiter) reaches the session as a Claude Code
task notification, not a prompt, so the hook never runs and the session never sees the gate's text.
This was observed in this session: no gate text on any notification turn, and the gate text present
on the human's prompt. It is not a missed restart: the session's settings carry the hook, and it
fired.

Every interactive session that talks to the human and wakes on background commands (the
orchestrator, aides, advisors) is presumably affected the same way. Only this advisor has been
checked.

## Impact

Messages to the human during their sleep hours, against the hard limits the human set (cvaq).
Any other context a `UserPromptSubmit` hook adds is missing from background turns in the same way.

## Follow-up

- Postmortem: [[postmortem-quiet-hours-didn-t-reach-replies-to-background-wa-v3b7|v3b7]].
- Fix: [[background-wakes-carry-the-session-s-prompt-context-quiet-ho-9s8u|9s8u]].
