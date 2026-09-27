---
id: 2mj9
title: Spike: process containment on Linux
opened: 2026-09-27
repos: [bridle]
changes: []
specs: []
needs: []
see: [9trg]
---

## What to find out

Does the v1 containment work on Linux as written, and is a cgroup per agent
the better mechanism there?

- `crates/bridle-daemon/src/containment.rs` runs
  `ps -axo pid=,ppid=,pgid=,lstart=`. Check that procps-ng on Ubuntu accepts
  it, and how `lstart` formats under different locales and time zones.
  `/proc/<pid>/stat` field 22 (starttime) is the locale-free source.
- Whether running each agent under `systemd-run --user --scope` (cgroup v2)
  catches double-forked daemons that the scan misses, and whether it works
  from a daemon that itself runs as a systemd user service with linger.

## Why it matters

The workforce host is Linux ([[docs/context/nuc-host|the NUC]]).
The v1 design deferred Linux containment to "when bridle moves to a VPS", and
that move is now the plan.

## Notes

Raised on 2026-09-27 while checking the NUC plan ([[docs/context/nuc-host|NUC host]]) against the agent-host design.
