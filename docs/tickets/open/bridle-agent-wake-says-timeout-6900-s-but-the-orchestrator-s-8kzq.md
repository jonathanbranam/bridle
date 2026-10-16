---
id: br-8kzq
title: bridle agent wake says 'timeout 6900 s' but the orchestrator's wait ends at 25 min
kind: bug
created: 2026-10-16
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

`bridle agent wake external:orchestrator` with no `--timeout` prints `waiting as external:orchestrator (pid N, timeout 6900 s)`, and its help says "the cap (and default) is 6900 s". But the CLI sends no `timeout_secs`, so the daemon applies `wake::POLL_TIMEOUT` (25 min, `crates/bridle-daemon/src/wake.rs:34`, via `clamp_timeout` in `server.rs:878`), and the wait exits 4 after 25 min. Seen on dalek on 2026-10-16: eight 25-minute timeouts in a row, each printing 6900 s.

Fix: make the CLI and the daemon agree. Either the CLI sends its 6900 s default, or it prints and documents the 25 min default. The role tells the orchestrator to "wait at the maximum timeout", so sending 6900 is probably the right one. Update api.md (the orchestrator's wait "defaults to a 25 min wait") and cli.md to match.

Cost of not doing it: an orchestrator that waits at the default re-arms every 25 min (more turns and tokens) and is misled about how long it waits. The workaround is to pass `--timeout 6900`.
