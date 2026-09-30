---
id: 6d5y
title: bridle restart waits on the old URL, so it reports failure when the port changes
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [k7mw]
---

## What happened

2026-09-30, after adding `[projects]` ports (k7mw) to `~/.bridle/config.toml`, the first
`bridle restart` on each machine reported failure although the daemon came back fine:

- The NUC (meta-notes, now on port 7402). The human:

  > file issue: bridle restart hung I think b/c bridle serve on the NUC was not busy and
  > restarted almost instantly. Not positive, but that seems the explanation: % bridle restart
  > restarting at b95c418: 1 agent to resume (stopping takes up to 35s)
  >
  > On a second restart it did work properly: [...] the daemon is back
  >
  > might be temporary or a race condition.

- dalek (bridle, from a registry port to 7401), run by the orchestrator:

  ```
  restarting at 6fb42f3: 2 agents to resume (stopping takes up to 35s)
  error: the daemon did not come back within 95s; see <workspace>/.bridle/daemon.log
  ```

  The daemon was up on `127.0.0.1:7401` and the Tailscale address, and both agents had resumed.

## Cause

`restart` in `crates/bridle/src/commands.rs` polls `health()` on the client it started with, per
its comment: "the same URL comes back with the new binary". A restart that picks up a new listen
port (a `[projects]` port, or a changed `[daemon] listen`) comes back somewhere else, so the
loop sees the daemon go down and never sees it return. Not a race: the second restart
worked because the port no longer changed.

## Direction (not triaged)

After the daemon goes down, re-resolve the endpoint (discovery: registry or `[projects]`) on each
poll instead of reusing the first URL. Small. The error also names `<workspace>/.bridle/daemon.log`,
which didn't exist on dalek (a daemon started in the foreground logs to its terminal).
