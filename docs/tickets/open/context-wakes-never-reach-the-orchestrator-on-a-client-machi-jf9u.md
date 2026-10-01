---
id: jf9u
title: Context wakes never reach the orchestrator on a client machine (NUC)
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [7d62, chvf]
---

## The ask


Reported by the NUC's orchestrator (m-2877, 2026-09-30): the meta-notes orchestrator on the NUC
reached ~182K context and got no `context` wake. It was launched properly
(`bridle session orchestrator --project meta-notes`), and `~/.bridle/orchestrator.pid`,
`orchestrator.session` and `~/.bridle/context/<session>` (182340) were all current. But
`bridle events --kind orchestrator.context` is empty and `bridle status` shows no context. The
daemon's workspace `.bridle/` (`/srv/shared/work/meta-notes-work/.bridle/`) has none of those files.

## What the code says (at 30b2b61)

The launcher (`crates/bridle/src/session.rs`), the note-session hook and the statusline write
under `discovery::bridle_home()`: `$BRIDLE_HOME`, else `~/.bridle`. The daemon's supervisor
(`crates/bridle-daemon/src/lib.rs`, `orchestrator_task`) reads from `overrides.bridle_home`,
else the same `bridle_home()`, and only runs when `[orchestrator] enabled`. So the likely causes,
to check on the NUC in this order:

1. `[orchestrator] enabled` is off for meta-notes. Then no supervisor runs at all. The log line
   "no orchestrator.pid: the orchestrator supervisor does nothing" would be absent too.
2. The daemon runs with a different `BRIDLE_HOME` (or `--bridle-home` override) than the
   orchestrator session, e.g. from how it was started on the NUC, so the two sides look in
   different directories.
3. The NUC's binary (0.3.0, built 19:06Z) predates a fix (chvf: client binaries lag).

## Also

- `~/.bridle/daemon.json` on the NUC names a `dotfiles-local` daemon (workspace `/home/jbranam`,
  port 37607), possibly stale.
- One `orchestrator.pid` per home can't serve two orchestrators on one machine; 7d62 (per-project
  pid files) covers that.

## Likely cause: the pid file's start time never matches (2026-10-01)

The NUC's handover relaunch test (m-2992, h-0002) came back partial: the new session started, but
the old one (pid 110487) was never stopped, and at 00:54:35Z the daemon filed "The orchestrator
session (pid 110487) is not running; found dead" while it was alive. One cause fits all three
symptoms: the supervisor never sees the session as alive.

- `write_pid_file` (`crates/bridle/src/session.rs`) records `ps -o lstart= -p <pid>` from the
  session's own environment.
- `RealProcs::is_alive` (`crates/bridle-daemon/src/orchestrator.rs`) compares it with
  `containment::start_time`, which reads `ps -axo pid=,ppid=,pgid=,lstart=` from the daemon's
  environment, and needs an exact string match.
- `lstart` is local time and follows `TZ` and `LC_TIME`. A daemon started differently from the tmux
  shell (systemd vs. a login shell; the NUC is on UTC) prints a different string. On Linux,
  procps also derives `lstart` from boot time plus jiffies, so it can drift by a second.
- If they never match, the session reads as dead. No context wakes are sent (this ticket), a
  "dead" incident is filed and a relaunch typed, and `signal` refuses to touch the "reused" pid.
  That's why the old session survived the handover.

To confirm on the NUC: compare the pid file's start time with `ps -o lstart= -p <pid>` run from
the daemon's environment (`/proc/<daemon pid>/environ`: TZ, LANG, LC_*).

Fix direction: compare start times as instants, not strings. Read them in a fixed locale and zone
(`LC_ALL=C TZ=UTC`) on both sides, or on Linux use `/proc/<pid>/stat` starttime. Allow a
second of slack, or share one helper so both sides produce the same string. Start-up/relaunch
path: schedule it with chvf after the trip (Sat 2026-10-03), not before.

### The human's diagnosis agrees (m-3010)

The human traced the relaunch to a timezone change. To the daemon, the session looked about 4 hours
old (the UTC/EDT offset), so it took the dead-session path and started a second orchestrator
beside the live one. The age and uptime arithmetic itself is consistent: it uses the pid file's
launch epoch against `Utc::now()`. What moves with the zone is the `lstart` string. Render the same
instant in another TZ and the identity check fails, which is the mismatch above. So it's one bug.
The human asks for both of these:

1. Process identity and age use one clock (UTC or monotonic), never a local-time string.
2. The daemon never starts a second orchestrator while the first is alive. Before relaunching
   after "found dead", it double-checks: if the recorded pid is alive and is a `bridle session`
   (or its child `claude`), but its start string doesn't match, file an incident and don't
   relaunch.

## Done when

The cause is found and fixed, and a client-machine orchestrator gets `context` wakes, with a test
for whichever split caused it. A timezone change (or a daemon running in a different TZ) doesn't
make a live session look dead, and a live session is never doubled; both are covered by tests.
