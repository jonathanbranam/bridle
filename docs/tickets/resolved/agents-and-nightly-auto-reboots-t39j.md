---
id: t39j
title: Agents and nightly automatic reboots
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: []
closed: 2026-09-30T05:12:44Z
---

## The question

What happens to running agents when the workforce host reboots itself for
updates?

## Why it matters

The NUC guide sets unattended-upgrades to `Automatic-Reboot "true"` at 04:00
when an update needs it. [[docs/design/agent-host/daemon#Restart and recovery|Restart and recovery]]: agents don't survive a
daemon restart. They are marked `lost`, and only roles with `autostart` or
`resume_on_restart` (the manager, by default) come back. A worker mid-turn at
04:00 loses that turn.

## Notes

Raised on 2026-09-27 while checking the NUC plan ([[docs/context/nuc-host|NUC host]]) against the agent-host design.

- Options: turn off automatic reboot and rely on Livepatch for kernel fixes; accept the
  loss; or have bridle drain before a reboot, e.g. hold a systemd shutdown
  inhibitor while turns are running.
- Run the daemon as a systemd user service in the foreground, not
  `serve --detach`, so systemd restarts it after a reboot.

## Resolution

Accept the lost-turn risk from nightly auto-reboot as-is. bridle doesn't run
on the NUC yet, so the systemd-inhibitor option (holding a shutdown inhibitor
while turns are running) is YAGNI for now. Revisit both this decision and the
resume policy ([[do-workers-resume-after-a-daemon-restart-2fkk|2fkk]]) when
bridle actually moves to the NUC.

Resolved 2026-09-28.
