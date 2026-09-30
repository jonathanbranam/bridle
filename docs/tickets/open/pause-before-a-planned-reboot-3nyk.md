---
id: 3nyk
title: Pause work before a planned reboot, resume after
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: [nuc-recovery-on-boot-4r3k]
see: [agents-and-nightly-auto-reboots-t39j]
---

## The ask

The human, verbatim (2026-09-30, via the advisor):

> if the reboot is every day, we could add functionality to our thermostat-like schedule to pause
> work say 5-10 min before a reboot, have orch prepare a handoff, and then resume on reboot. That
> doesn't feel as urgent as these other asks, though, and a standing rule for the nuc orchestrator
> could handle it in the short term. But I'm OK for this week either (1) disable reboot or (2) let
> it reboot and see what happens. When I'm traveling I'll disable reboot though for safety.

Not urgent. It builds on t39j (resolved: accept the lost turn) now that bridle runs on the NUC.

## Facts

unattended-upgrades reboots at `Automatic-Reboot-Time` (04:00) **only when an installed update asks
for it** (`/var/run/reboot-required`: kernel, libc, systemd and the like), not every day. The human
wants to try different settings over a few weeks.

## Track the reboots (the human, 2026-09-30)

> I'd love to track that. the nuc is a new install so there won't be any history with the reboots
> yet; it would be interesting to know how often they happen in reality; also a weekly reboot might
> be a good thing to schedule as well.

- Record each night a reboot is pending (`/var/run/reboot-required` and `.pkgs`) and each boot
  (`journalctl --list-boots`; the journal is persistent). A few weeks of that shows how often.
- As of 2026-09-30 12:30 UTC: 5 boots since the install on 2026-09-29 (all by hand, during setup),
  and a reboot is **pending** (`gnome-shell`, flagged 06:58 UTC). The NUC is on UTC, so the 04:00
  reboot is midnight Eastern, i.e. tonight's, unless it's turned off first (br-276e).
- A **weekly scheduled reboot** (e.g. Sunday 04:00) is worth trying alongside, once 4r3k makes a
  reboot recoverable.

## Shape

- Short term: a standing rule for the NUC orchestrator: if `/var/run/reboot-required` exists, wind
  down before 04:00 and write a handover.
- Later: a scheduled window in the budget schedule ("thermostat"), e.g. 03:50 to 04:10: hold new
  turns, let running ones end, ask the orchestrator for a handover; after boot (4r3k) everything
  resumes. Maybe only on nights when a reboot is pending.
