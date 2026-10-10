---
id: 7ufd
title: Self-upgrade at most every few hours, batching the landings; critical fixes go through at once
kind: feature
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [yfv5, 3nyk, e35h, y455]
tasks: [br-7ufd]
closed: 2026-10-10T01:32:31Z
---

## The ask

The human, 2026-10-09 ~6:15 PM ET, verbatim (to the aide):

> how many restarts has happened today?
>
> dameon restarts? seems like that is happening much too often
>
> feels like it is upgrading after every ticket or two; almost every ticket

Then, after the aide laid out three options (batch with a minimum interval; nightly window only;
turn `self_upgrade` off):

> we need to batch them and also wait several hours. I agree with (1). yes, critical fixes can be pushed through

## The facts (the daemon's events, 2026-10-09)

- bridle's daemon started 18 times on 10-09 (ET, by ~6 PM), 17 of them self-upgrades
  (`upgrade.built`); 13 more candidates were skipped as docs-only. Earlier days: 10-05 12 starts,
  10-06 4, 10-07 16, 10-08 10 (`bridle events --kind daemon.started`).
- Cause: `[daemon] self_upgrade = true` in bridle's `.bridle/config.toml`. The CI watcher's
  one-minute tick upgrades to any newer green main commit that changes the binary's inputs, with
  no minimum gap and no batching (`docs/design/agent-host/daemon.md`, "Automatic
  upgrade").
- Each restart drains every agent's turn, resumes the workers, ends interactive sessions' waits
  (exit 6; they re-arm by hand), and loses the session registry
  ([[interactive-sessions-survive-a-daemon-restart-the-registry-i-e35h|e35h]]).

## The ask, as understood (aide)

1. The automatic upgrade waits several hours after the last one before it upgrades again, then
   takes the newest green commit, so every landing in between goes in as one batch. The interval
   is a config value (the human: "several hours"; 3 h is a starting point to confirm).
2. A critical fix can go through at once: an explicit `bridle restart --upgrade` (the human or the
   orchestrator) ignores the interval, as today. Whether a critical-priority task landing should
   trigger it by itself is for the design.
3. Out of scope: the nightly maintenance window
   ([[one-scheduler-for-timed-actions-scheduled-messages-hrcn-nigh-yfv5|yfv5]],
   [[pause-before-a-planned-reboot-3nyk|3nyk]]). This is the small step that can be built before it.

## Resolution

br-7ufd landed 2026-10-09 ~9:32 PM ET (41d6e0fd). `[daemon] self_upgrade_min_interval` (default
3h; `0s` restores the old behaviour): the automatic self-upgrade holds a newer green commit until
the interval has passed since the newest stored `upgrade.built` event, so the landings in between go
in as one batch, and the wait survives restarts. An explicit `bridle restart --upgrade` ignores
the interval (the critical-fix path). A critical-priority landing does not trigger an upgrade by
itself (deferred; stated in daemon.md). The human's reminder to turn `self_upgrade` back on is
br-x7fx.
