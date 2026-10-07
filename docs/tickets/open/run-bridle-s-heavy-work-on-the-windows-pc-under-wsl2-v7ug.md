---
id: v7ug
title: Run bridle's heavy work on the Windows PC under WSL2
kind: feature
opened: 2026-10-07
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [ur9v, b7cz, npj2]
tasks: []
---

## The ask

The human, 2026-10-07 ~7:20 PM ET, via aide (m-6585), answering whether to plan moving heavy
bridle work off dalek (the Intel Mac) to the Windows PC under WSL2:

> yes plan the work to run on the windows pc - it'll be a while maybe the weekend until I can get
> to that though.

The research and the choice (WSL2 with Ubuntu, not dual boot) are in
[[the-windows-gaming-pc-as-a-bridle-host-without-wiping-window-ur9v|ur9v]], with the human's
answers (Windows Home, BitLocker off, plenty of disk, gaming means bridle stops, a big red / big
green button). Why: Rust builds on dalek take ~10 minutes and macOS scanning adds load
([[build-cost-on-the-laptop-b7cz|b7cz]], research npj2 in flight). The NUC is too small for
bridle's own builds (`docs/context/nuc-host.md`).

## What to plan

Split it into tasks bridle's workforce can do now and to-dos for the human:

- **The human's hands-on steps** (maybe the weekend): each a `--for-human` to-do, never a blocker
  on the queue. Install WSL2 + Ubuntu, `.wslconfig`/`wsl.conf` (systemd, idle timeout, mirrored
  networking), Tailscale, the Task Scheduler start-up task, power and Windows Update settings, and
  note the CPU and RAM (still unknown). A setup guide they can follow step by step is the deliverable
  bridle writes; it should reuse the NUC guide's shape.
- **Bridle work, if any:** what has to change for a WSL2 host (likely little: it's Linux; check
  `bridle systemd install` under WSL's systemd, discovery across machines, and what "heavy work"
  means: which projects or which agents move, and how dalek's sessions reach that daemon).
  Decide whether bridle's own repo moves or only builds do (e.g. remote builds); recommend.
- **The button:** two desktop shortcuts (`wsl --shutdown`, and the start-up task) as the first form.
- **A name** for the PC (`docs/context/naming.md`).

Out of scope until the human has the PC set up: anything that needs the PC to exist.
