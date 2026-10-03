---
id: wz42
title: "A big red button: pause bridle on a machine, in levels, by hand or when a game starts"
kind: feature
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [3nyk, ur9v, b7cz, m7wn]
tasks: []
---

## The ask


A feature that needs design first (no task). The human, verbatim (2026-10-03, via the advisor),
after the Windows PC ticket ([[the-windows-gaming-pc-as-a-bridle-host-without-wiping-window-ur9v|ur9v]]):

> Yeah, that's the idea. And actually, I think there's a there's another concept here that we've
> talked about that could probably get a little more planning around it for this use case as well.
> And that's you know, big red button. The idea is like I run into this on my work laptop too.
> Dalek, no, sorry, not my work laptop. On Dalek, my personal laptop, it's it's under a lot of
> strain building Bridle, and like Vim hardly loads, which seems really crazy to me. But you know, I
> could use a big red button, and really, what that doesn't need to like. There's a there's a
> couple of levels of this, right? One of them is just probably stopping. Builds of Bridle, like, so
> that means just pausing the worker and manager, and for the for my laptop, that's probably enough
> for Dalek to get going again, and and I can just turn it all back on. So that's not as extreme for
> the gaming PC, you know. Oh, for update. So the other idea here is for scheduled or unscheduled
> like reboots and things. For scheduled ones, like on the NUC, we should schedule Bridle. Bridle
> should interact with the operating system and check when a scheduled update is due and shut
> itself off before that happens and then you know be restarted when that's finished. Same thing for
> Windows. I don't know if you can... interact with Windows from WSL2 or whatever, but you know we
> could also write a little Windows script that interacted and told us when things were scheduled.
> Probably PowerShell would do that, and you know so we could shut off Bridle around that. Now that
> PC is just literally off, so you know nothing is getting updated now, and I don't I don't know
> what my auto update settings are, but. It probably needs a couple, probably needs a whole bunch of
> security patches by now. Oh, and my other thought was like, again, may need some Windows
> integration, but you know, we could just detect when somebody starts Steam or the Epic Game Store
> or some sort of a, you know, some sort of a game or something and then kind of quiet things down
> on Bridle and any work that's being done. We wouldn't necessarily have to shut everything off
> instantly, but we could quiet things down and then maybe do a graceful exit.

("Bridle" was dictated as "Bridal" in places; fixed here.)

## Levels (the human's idea, shaped by the advisor)

1. **Pause the building.** Pause the workers and the manager so builds stop; the machine gets its
   CPU back. Enough for dalek, where building bridle makes even Vim slow to load. One action to
   pause, one to resume.
2. **Stop bridle on this machine.** Wind down, then stop the daemons (and on the Windows PC, WSL),
   for a game or anything heavy. Bridle's restart resilience means little is lost.

Close to level 1 today: `bridle usage budget hold [--for D | --until HH:MM]` / `release` idles
the account's work for the current daemon (wind down: running turns finish, no new ones). It's
named and documented as a budget tool, applies to one daemon only (cross-daemon hold is planned),
and isn't a "button". See also [[build-cost-on-the-laptop-b7cz|b7cz]] (build cost on the laptop).

## Triggers

- **By hand:** a big red / big green button (a command, a desktop shortcut on Windows, maybe the
  web UI or the phone).
- **Before an OS update or reboot:** bridle checks the OS for a scheduled update, winds down and
  stops before it, and comes back after. On the NUC that's
  [[pause-before-a-planned-reboot-3nyk|3nyk]] (`/var/run/reboot-required` before the 04:00
  reboot). On Windows, a small PowerShell script can read Windows Update's pending restart and
  scheduled time and tell bridle; WSL can run Windows programs (`powershell.exe`) directly.
- **When a game starts:** detect Steam, the Epic Games Store or a running game on the Windows PC,
  quiet bridle down (no new turns), then exit gracefully. Needs a Windows-side watcher (Task
  Scheduler or a small PowerShell loop) telling bridle; gentle, not an instant kill.

## Also noted

The Windows PC has been off, so it likely needs a lot of security updates before anything else;
its update settings are unknown.
