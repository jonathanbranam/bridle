+++
id = "br-nxkj"
title = "A second 'bridle session orchestrator' overwrites the live session's orchestrator.pid"
kind = "bug"
state = "integrated"
created_at = "2026-11-14T11:06:11.459Z"
updated_at = "2026-11-14T12:50:58.745303Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:nxkj",
]
branch = "bridle/nxkj"
commit = "c0398f607892d599fcc51f70f354c75d74731369"
summary = "orchestrator() in crates/bridle/src/session.rs now refuses (bail naming the pid, pid file untouched) when orchestrator.pid names a live process with the recorded start; stale/dead/mismatched files are overwritten. Reuses the daemon's parse_pid_file and liveness test via a new pub fn live_recorded_pid in orchestrator.rs (additive, re-exported from lib.rs; no behaviour change). Tests in session_test.rs; one line in cli.md; CHANGELOG Fixed."
ticket = "nxkj"
+++

docs/tickets/open/a-second-bridle-session-orchestrator-overwrites-the-live-ses-nxkj.md

submitted by external:orchestrator

Filed by orchestrator@dalek for the human, relayed by orchestrator@vesta (m-0652 on dalek-home). Found while investigating the aide waiter gap (submitted alongside, same message).

## What happened

At 2026-11-14 11:00:57 UTC something on vesta ran a second `bridle session orchestrator` while the live one (launcher pid 956878) was running. It overwrote `~/.bridle/orchestrator.pid` with pid 1009685 and lstart `@1794654057`, and its claude exited 1 at 11:01:00. The daemon then flagged a pid/poll disagreement (m-0294 on vesta-home). The vesta-home daemon log shows no relaunch at that time, so the source is unknown.

## The ask

- A second launcher must not overwrite the pid file of a live orchestrator session: check that the recorded pid (with its lstart) is still alive and refuse, or exit, instead.
- Find what started the second launcher, if the logs allow.

## Thread

### note · external:orchestrator · 2026-11-14T11:06:11.460Z
submitted by external:orchestrator

### note · external:orchestrator · 2026-11-14T11:06:57.876Z
readied by external:orchestrator

### note · external:orchestrator · 2026-11-14T11:08:14.778Z
Source of the 2026-11-14 11:00:57 UTC launch on vesta found: the human ran it by mistake (human via orchestrator@vesta: 'yes, that was my fault. typo'). The bug still stands: an accidental second launch must not overwrite a live session's pid file and push the supervisor into recovery mode. The 'find what started it' part of the ask is done.

### note · external:orchestrator · 2026-11-14T11:11:15.865Z
More detail, human via orchestrator@vesta: the human pressed up-arrow then Enter in the orchestrator's tmux pane by accident. The recalled line was 'bridle session orchestrator --project vesta-home', which is in the shell history because the supervisor's relaunch types that line into the pane. So every relaunch leaves a live re-launch one keystroke away. Fix to consider: the launcher refuses to start, and leaves the pid file untouched, when the pid file names a live session.

### note · agent:nxkj · 2026-11-14T12:50:22.543Z
built: e406d62e (branch bridle/nxkj)

### note · agent:nxkj · 2026-11-14T12:50:22.580Z
done: second orchestrator launcher refuses on a live pid file; e406d62e, just check (env -i) exit 0, 2006 tests passed

### note · agent:manager-2 · 2026-11-14T12:50:43.220Z
integrated: c0398f607892d599fcc51f70f354c75d74731369 (branch bridle/nxkj)

### note · agent:manager-2 · 2026-11-14T12:50:58.745Z
cleanup: removed agent nxkj, origin branch bridle/nxkj, branch bridle/nxkj
