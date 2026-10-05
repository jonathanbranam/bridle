---
id: v3b7
title: "Postmortem: quiet hours didn't reach replies to background wakes"
kind: research
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [yy88, 9s8u, cvaq, u6w9, cr7t, mvtz]
tasks: []
---

The kind is `research` until a `postmortem` kind exists ([[add-a-postmortem-ticket-kind-the-full-write-up-after-an-inci-cr7t|cr7t]]).

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

## Summary

The human's quiet hours (cvaq) limit what interactive sessions say to the human: at most 3
sentences, a nudge back to their work, and no new threads. The limits come from a hook that runs
only when the human sends a message. Sessions also take turns when a background command ends,
mostly their wake waiter delivering a message or task change. Those turns never got the limits. So
on 2026-10-04 the bridle advisor kept sending full daytime replies after 21:30 ET, until the human
noticed at about 22:35 and asked whether quiet mode was reaching it at all. Incident:
[[interactive-sessions-reply-in-quiet-hours-the-focus-gate-nev-yy88|yy88]].

## Impact

- At least two long replies (about 21:56 and 22:28 ET) to a human who had asked for quiet, and very
  likely more from other sessions. Only this advisor's were checked.
- Trust: quiet hours are a hard limit the human set for their own wellbeing, and sessions can't keep
  it in the very turns where they speak unprompted.
- No work was lost.

## Timeline (2026-10-04, ET)

- 21:00: a system message says the advisor's context is at 150k, before quiet hours.
- 21:30: "weekday-sleep" quiet hours begin.
- ~21:56: the daemon restarts. The advisor's wait ends (reported as a timeout), and the advisor
  replies in full, in the daytime style, about the restart.
- 22:28: the orchestrator's message (link tickets by URL) wakes the advisor. It replies with three
  links and "I'm waiting for your message".
- ~22:35: the human writes. The `UserPromptSubmit` hook adds the QUIET HOURS text, and the advisor
  sees it for the first time that night. It answers within the limits, and the cause becomes clear
  from what it has and hasn't seen.

## How it happened

1. `bridle session` installs `bridle focus gate` as a `UserPromptSubmit` hook, and only there
   (`FOCUS_GATE` in `crates/bridle/src/session.rs`). The design (cvaq) assumed every reply follows a
   prompt from the human.
2. Interactive roles are built to wake without a prompt: the waiting loop in every role prompt is
   one background `bridle agent wake` (or `wait-for-wake`). When it ends, Claude Code hands the
   session a task notification, which isn't a prompt, so `UserPromptSubmit` doesn't run.
3. The role prompts make quiet hours conditional on seeing the gate: "when the prompt's context
   says QUIET HOURS". With no gate text, a session follows its daytime habits and tells the human
   each update.

## The hooks and rules that allowed it

- **Hooks:** the gate is on the one event that background turns skip. Other hooks have the same
  question and haven't been checked. `Stop` (`bridle focus reply`, which records when the session
  finished replying, for u6w9) presumably also fires after background turns. If so, it records
  replies that answered no prompt from the human.
- **Rules:** none says "check the time yourself", and that's right: the gate exists so sessions
  don't each work out quiet hours. But no rule or checklist asks, when a hook is added, "does this
  also need to reach turns the human didn't start?"

## What went well

- The human noticed and asked directly. The session could tell from its own turns which ones
  carried the gate, so the cause was clear within one exchange.

## Learnings

1. **Interactive sessions have two kinds of turn: the human's prompt, and background wakes.** Any
   context meant to shape what a session says to the human has to reach both.
2. **Every hook needs that question asked when it's added**, the human's point: "every time we're
   adding a hook, whether that hook also should be included as part of background messages."
3. **A limit that depends on the agent seeing a notice fails silently when the notice is missing.**
   Where it can, the channel that wakes the session should carry the notice itself.

## Actions

| Action | Where | State |
|---|---|---|
| Wakes carry the session's prompt context (quiet hours first); a hook audit; a rule for new hooks | 9s8u | filed |
| A `postmortem` kind and folder; move this there | cr7t | filed |
