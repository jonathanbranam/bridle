---
id: zm95
title: Shutdown hangs while a client holds the event stream open (bridle tui)
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: [sed3]
---

## What happened

The human, verbatim (2026-09-28): "Bug to file: in meta-notes, bridle serve received
shutdown but never shut down (I didn't wai too long) was held open seemingly by 'bridle
tui' running. When I quit tui, it stopped immediately."

## Notes

Read from the code at 4e06628; not reproduced.

- `crates/bridle-daemon/src/lib.rs` (the shutdown task): after stopping agents, flushing
  tasks and removing the registry entry and `daemon.json`, it signals axum's
  `with_graceful_shutdown` and then awaits `serve_task`. Graceful shutdown waits for
  every open connection to finish.
- The SSE handler (`crates/bridle-daemon/src/server.rs`, the events stream) ends only
  when its broadcast receiver reports `Closed`, which needs every sender dropped. The
  daemon's own emitter still holds one, so the stream never ends on shutdown, and
  `serve_task.await` waits for the client to disconnect. The TUI subscribes to this
  stream (`events_stream`); so does anything else following events.
- `daemon.json` is already removed by then, so the daemon looks stopped to discovery
  while the process lingers.
- Likely fix: end SSE streams when shutdown starts (select on the shutdown signal in the
  stream), and/or bound `serve_task.await` with a timeout.
