---
id: eadm
title: "Keep dalek awake while it's open: a toggled check every ~30 min that tells the orchestrator when nothing is preventing sleep"
kind: feature
opened: 2026-10-10
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [zcqv, tv8r]
tasks: [br-eadm]
---

## The ask

The human, 2026-10-10 ~1:00 PM ET, verbatim (to the aide):

> Okay, I rebooted my laptop and forgot to start caffeinate, so it looks like it's been going to
> sleep here and there. I started running that again, so that's interesting and would explain a
> few things I've seen since my last reboot.
>
> I don't know if there's a different way to keep this awake, but let's add this to a monitor
> that runs infrequently and sends a message to the orchestrator if it's not running. That should
> be a toggle. We need to be able to toggle that on and off because there'll be times when I don't
> want caffeinate running. Infrequently, like 30 minutes, I don't know, something like that.

## Facts (2026-10-10)

- The human runs `caffeinate -si` by hand after each reboot (pid 78284, started ~1:00 PM ET
  today). Nothing restarts it after a reboot; Claude Code's own `caffeinate -i -t 300` calls only
  cover a working session.
- `-s` holds off system sleep only on AC power; `-i` only idle sleep. Neither stops clamshell
  sleep, and the human wants a closed laptop to sleep (ticket zcqv, comment c2: "If I close the
  laptop, it should sleep."). So this is about an **open** dalek sleeping when idle.
- `pmset -g assertions` shows whether anything holds `PreventSystemSleep` /
  `PreventUserIdleSystemSleep`, and which pid; that is a better check than looking for a process
  by name.
- Other ways to stay awake (for the design): a launchd user agent that runs `caffeinate -si` at
  login with KeepAlive (survives reboots, no human step), or `sudo pmset -c sleep 0` (never sleep
  on AC, persistent, no process; ticket tv8r to-do br-1b47 used it for a trip). Each would make the
  monitor a safety net rather than the fix.

## The ask, itemised

1. A check about every 30 minutes (the human: "Infrequently, like 30 minutes") that dalek is being
   kept awake, and a message to the orchestrator when it isn't (which may relay to the human).
2. A toggle, on and off, for times the human doesn't want caffeinate running; off means no check
   and no message. Where it lives (config, a `bridle` command) is for the design; it should be
   easy for the human and visible in `bridle status`.
3. Answer the human's "I don't know if there's a different way to keep this awake": compare the
   launchd agent and `pmset` options above with the monitor alone, and recommend one.

## Decision (the human, 2026-10-10 ~11:00 AM ET, to advisor product-manager)

No launchd agent for caffeinate. The human's words: "No, ... definitely not. Caffeinate is a human
decision, and I do not necessarily want it always running, so it is not a priority. It only
happens at reboot. Yeah, it can wait."

So: keeping dalek awake stays the human's call, made by hand. The task stays pending, unscheduled,
for the next PdM review; if it's built, it only notices and tells, never starts caffeinate itself.

The human, a moment later (verbatim):

> So, that ticket, I should probably review. We can come back to this later. It's just that I'd
> like to know if I forgot to run it, but we don't want to run a constant check or a poll or
> anything. I just want to have a reminder that it's not running occasionally, because I might
> just forget, but I want to be free to turn it off whenever I want to.

So the ask is an occasional reminder, not a ~30 min poll: something that already happens anyway
(for example the orchestrator's start-up or morning summary, or the aide's briefing) mentions
when nothing is keeping dalek awake. The human turns caffeinate off freely; the reminder never
nags or acts. Waits for the human's review of this ticket.
