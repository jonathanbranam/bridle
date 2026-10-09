---
id: fpde
title: Restart on Linux execs '<path> (deleted)' after the binary is replaced
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: []
kind: bug
tasks: [br-fpde]
---

## The ask


Reported by the NUC's orchestrator for the human, 2026-10-01 (m-3149). `bridle restart` on the
NUC (Linux) failed after `just install` replaced the binary under a running 0.3.x daemon
(meta-notes, port 7402). The daemon logged, verbatim:

```
WARN bridle_daemon::server: restart requested via POST /v1/restart principal=human
error: restart: exec /home/jbranam/.cargo/bin/bridle (deleted) failed: No such file or directory (os error 2)
```

The daemon then exited, and the human had to start it by hand.

## Cause

On Linux, `std::env::current_exe()` reads `/proc/self/exe`. Once `cargo install` replaces the
file, that link names the old inode with ` (deleted)` appended. `exec_self` and `roll_back`
(`crates/bridle-daemon/src/lib.rs:216`, `:230`) exec that literal path. The self-upgrade path
calls `current_exe()` after its build too (`upgrade.rs:187`, the `serve --check` preflight;
`upgrade.rs:167`, the rollback stash). So a self-upgrade on Linux breaks the same way.
macOS returns the real path, which is why the laptop never hit this.

## Wanted

- Resolve the binary's path once at daemon start-up, before anything can replace it, and use
  that path for every re-exec, preflight and rollback. Also strip a trailing ` (deleted)` as a
  fallback.
- If the exec still fails, stay up and report the error rather than exiting. An unattended
  machine must never lose its daemon to a restart.
- Add a unit test for the suffix handling. A Linux-only test that replaces the binary is welcome
  if it's cheap.

## Status

- 2026-10-01: the path fix landed as 8edb6ed (br-7b79). The path is resolved once and
  ` (deleted)` is stripped for exec, rollback and preflight.
- Still open: staying up when the exec fails. Today a failed exec still exits after a clean
  shutdown. Fixing that needs an in-process restart or a supervisor, so it was out of
  br-7b79's scope.
- Also still open: `agent_path()` in `supervisor.rs` (about line 2952) still calls
  `current_exe()`. On Linux, after the binary is replaced, agents spawned from then on could be
  given the ` (deleted)` path. It should use `crate::exe_path()` too.
- `bridle/self-upgrade` (dbdc47f) uses `exe_path()` in `server.rs`'s release upgrade as well;
  rebased 2026-10-01.
