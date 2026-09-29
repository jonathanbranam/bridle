---
id: sed3
title: Log shutdown requests at WARN
opened: 2026-09-28
repos: [bridle]
changes: []
specs: []
needs: []
see: []
---

## The ask

The human, verbatim (2026-09-28):

> low priority ticket: When bridle serve receives a shutdown request, have it print that
> out as a WARN level log so that I can see it.

## Notes

- `POST /v1/shutdown` (`shutdown` in `crates/bridle-daemon/src/server.rs`) sends on
  `shutdown_tx` without logging. SIGINT and SIGTERM (`Signals::listen` in
  `crates/bridle-daemon/src/lib.rs`) do the same; only SIGHUP logs ("received SIGHUP;
  ignoring", at INFO).
- Saying what triggered it (the API and its principal, or which signal) would tell the
  human who shut the daemon down.
- Low priority (the human).

## Resolution

Shutdown via `POST /v1/shutdown` (with the caller principal) and via SIGINT/SIGTERM is logged at WARN by the daemon (`server.rs` `shutdown`, `lib.rs` `Signals::listen`).
