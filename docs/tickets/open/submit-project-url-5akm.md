---
id: br-5akm
title: ticket submit --project files on the agent's own daemon when BRIDLE_URL is set
kind: bug
created: 2026-11-26
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

`bridle ticket submit --project bridle-ui ...` run by agent pm-1 (bridle, dalek) on 2026-11-26 filed the submission on bridle's own daemon (br-6tvm, since dropped) instead of bridle-ui's. Suspected cause: agents have `BRIDLE_URL` set, and the CLI's discovery order puts `--url`/`BRIDLE_URL` above `--project`, so `--project` is silently ignored. The same command from the orchestrator session (no BRIDLE_URL) reached bridle-ui correctly (ui-32at).

Fix: an explicit `--project` naming a different project than `BRIDLE_URL`'s daemon must win, or the command must fail and say why. Never file silently on the wrong daemon.

Verify: with BRIDLE_URL pointing at project A's daemon, `bridle ticket submit --project B` files on B (or errors clearly); a test covers it.
