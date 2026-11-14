---
id: br-nxkj
title: A second 'bridle session orchestrator' overwrites the live session's orchestrator.pid
kind: bug
created: 2026-11-14
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

A second `bridle session orchestrator` must not overwrite the pid file of a live orchestrator session. In crates/bridle/src/session.rs, `write_pid_file` (called near line 313 for `orchestrator.pid`) should first read the recorded pid and lstart; if that process is still alive and its start time matches, refuse (exit with a clear message naming the pid) instead of writing. A stale file (dead pid or mismatched start) is still overwritten.

What happened (orchestrator@dalek for the human, relayed by orchestrator@vesta, m-0652 on dalek-home): on vesta at 2026-11-14 11:00:57 UTC something ran a second `bridle session orchestrator` while the live one (launcher pid 956878) was running. It overwrote ~/.bridle/orchestrator.pid with pid 1009685 (lstart @1794654057); its claude exited 1 at 11:01:00. The daemon then flagged a pid/poll disagreement (m-0294 on vesta-home). The vesta-home daemon log shows no relaunch then, so the source is unknown.

## Scope

- The guard above, with the parse and liveness helpers the daemon already has (crates/bridle-daemon/src/orchestrator.rs: `parse_pid_file`, PidRecord); reuse, do not duplicate, if the crates allow it.
- Finding what started the second launcher: only if the logs on this machine allow; otherwise say so on the thread. Not blocking.
- No wire or project-file change, no migration. Docs: where `bridle session` is described in docs/design/agent-host/ or cli.md, one line.

## Acceptance

Tests (crates/bridle/tests/session_test.rs style): live recorded pid -> second launcher refuses and leaves the file untouched; dead or mismatched pid -> overwritten. `just check` with BRIDLE_* env unset. Model: Sonnet.
