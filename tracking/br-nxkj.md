+++
id = "br-nxkj"
title = "A second 'bridle session orchestrator' overwrites the live session's orchestrator.pid"
kind = "bug"
state = "pending"
created_at = "2026-11-14T11:06:11.459Z"
updated_at = "2026-11-14T11:06:29.387531Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
