---
id: ur9v
title: "The Windows gaming PC as a bridle host without wiping Windows: WSL2 or dual boot"
kind: research
opened: 2026-10-03
repos: [bridle]
changes: []
specs: []
needs: []
see: [chvf, tv8r]
tasks: []
---

## The ask


For the plan, not built yet (no task). The human, verbatim (2026-10-03, via the advisor):

> Sorry, I'm behind on messages, and I gotta probably get on to some other things here for today.
> But I want to add another thing that to get in the plan is something we should do. Why I want to
> do coming up, and that is to I have a Windows machine here. I think it's running Windows 11, and
> I doesn't it doesn't even run. Nobody uses it, so. I'd like to get that set up with Bridal. It
> can take a significant amount of the load off of Dalek. But my question here is, of course, is
> that, how's that going to work? So nobody uses it, but I don't want to wipe the operating
> system. So and I'm not sure if I really want to mess with WSL or not. So, yeah, do a little bit
> of research and make some suggestions on what I could do. I'd be okay probably with a, a dual
> boot option. Basically, it's a, it's a gaming PC that my kids use sometimes and I use sometimes.
> But we don't, we just don't use it. Like 90% of the time, it's just sitting there doing nothing.

("Bridal" is bridle.)

## Constraint: bridle needs Unix

Bridle doesn't run on native Windows: the daemon uses Unix process groups and signals for
containment (`containment.rs`), Unix paths and sockets (`paths.rs`, `discovery.rs`), and tmux for
interactive sessions. So the PC needs Linux one way or another. Claude Code itself runs anywhere.

## Options (advisor's research, from general knowledge; verify on the machine)

1. **WSL2 with Ubuntu (recommended first).** Linux in a lightweight VM inside Windows. No wipe,
   no repartitioning, no reboots to switch; the kids keep gaming on Windows.
   - Turn on systemd (`/etc/wsl.conf`: `[boot] systemd=true`), so `bridle systemd install` works
     as on the NUC.
   - Keep it running: WSL stops an idle VM, so set `vmIdleTimeout=-1` in `%UserProfile%\.wslconfig`
     and start it at boot with a Task Scheduler task ("run whether the user is logged on or not")
     that runs `wsl.exe` with a long-lived command.
   - Reach it from dalek: `networkingMode=mirrored` in `.wslconfig` (Windows 11) makes WSL share
     the PC's network, so SSH to the PC reaches Linux; or Tailscale inside WSL.
   - Share nicely: cap WSL's RAM and CPUs in `.wslconfig` (`memory=`, `processors=`) so a game
     still runs; bridle's budget hold can wind workers down when the PC is in use.
   - Windows side: power plan never sleeps; Windows Update active hours set so it doesn't reboot
     mid-run (the NUC's 04:00 reboot lesson, tv8r).
   - Cost: Windows' overhead stays; a Windows reboot (updates, a kid) stops bridle until it comes
     back; more moving parts (Task Scheduler, .wslconfig).
2. **Dual boot, Linux on its own second SSD.** Full Linux, same as the NUC. A separate drive
   avoids shrinking the Windows partition; pick the OS in the boot menu.
   - Cost: the PC is either a bridle host or a gaming PC, never both: someone gaming means
     rebooting into Windows and bridle is down; the boot menu default decides what happens after a
     power cut. BitLocker and Secure Boot may need attention.
3. **A Hyper-V VM** (Windows 11 Pro only): like WSL2 with more control and more setup. No
   advantage here.
4. **Native Windows:** not possible today (above).

## Recommendation

Start with **WSL2**: it fits "don't wipe it, the kids still use it", and the PC is idle 90% of the
time. Move to a second-SSD dual boot only if WSL proves unreliable as an always-on host.

## To find out on the PC

Windows edition (Home or Pro), CPU and RAM (how much it can take off dalek), free disk space,
whether BitLocker is on, and that it's on wired network.

## Also

- It needs a proper name, not "Windows" ([[docs/context/naming|naming]]: a type is not a name).
- Client-machine updates apply as for the NUC
  ([[client-machines-stay-current-with-bridle-workflow-and-daemon-chvf|chvf]]).
