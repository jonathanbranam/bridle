---
id: 9s8u
title: "Background wakes carry the session's prompt context: quiet hours, and every hook that should apply to them"
kind: feature
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [yy88, v3b7, cvaq, u6w9, 75h2]
tasks: []
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

What happened, and why: the incident
[[interactive-sessions-reply-in-quiet-hours-the-focus-gate-nev-yy88|yy88]] and the postmortem
[[postmortem-quiet-hours-didn-t-reach-replies-to-background-wa-v3b7|v3b7]]. In short, the focus
gate is a `UserPromptSubmit` hook. Turns that start from a background command (a session's wake
waiter) never run it, so those turns don't get quiet hours.

## Proposals

- **P1. The wake prints the gate.** When `bridle agent wake` and `bridle orchestrator
  wait-for-wake` return, for a message, a task change or a timeout, they first print what `bridle
  focus gate` would add at that moment, using the same function (`focus::gate`). That way the text
  lands in the output the session reads. They print nothing extra outside quiet hours. This is the
  fix for yy88, and it's small: the CLI already runs on the machine with `~/.bridle/config.toml`.
- **P2. The gate's wording for wakes.** On a background turn the human hasn't written, the current
  first sentence ("nudge the human back to their work") doesn't fit. On a wake the gate says
  instead: in quiet hours, don't message the human about this unless it's urgent; deal with it
  quietly (tickets, `bridle send` to agents, re-arm the waiter), or save it for the end of quiet
  hours. Restarting the waiter is always allowed.
- **P3. A hook audit.** List every hook bridle installs in interactive sessions, and decide for each
  whether background turns need it:
  - `SessionStart`: `bridle orchestrator note-session` and `bridle session note`;
  - `UserPromptSubmit`: `bridle focus gate`;
  - `Stop`: `bridle focus reply`;
  - the workflow layers' hooks, `PreToolUse` `bridle arch-guard` among them.

  Check in particular whether `Stop` runs after background turns. If it does, u6w9 counts replies
  the human never asked for, and `bridle focus reply` should ignore those.
- **P4. A rule for bridle's own development** (the human: "every time we're adding a hook, whether
  that hook also should be included as part of background messages"). In bridle's
  `.bridle/rules/`, for the roles that build bridle: a hook added for an interactive role must say,
  in its doc comment and in `docs/design/cli.md`, whether it also applies to background wakes. If it
  does, its context is wired into the wake's output as in P1.
- **P5. One place for a session's per-turn context.** If P3 finds more than the gate, the wake
  runs a single function (call it `session_context`) that both `UserPromptSubmit` and the wake
  output use, so the two can't drift. Not needed if the gate is the only one (YAGNI).

## Questions

- **Q1.** Does any Claude Code hook fire when a background task's notification reaches the model?
  If one does, it could carry the gate instead of P1. Unverified; check
  `docs/spikes/01-stream-json-findings.md` and Claude Code's hook list. Recommendation: P1 anyway.
  The wake is ours, and it also covers `claude -p` sessions.
- **Q2.** In quiet hours, should a session reply to the human at all on a background turn, beyond
  something urgent? Recommendation: no (P2). The human sees it in the morning.
