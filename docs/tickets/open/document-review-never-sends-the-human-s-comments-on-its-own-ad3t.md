---
id: ad3t
title: "Document review never sends the human's comments on its own: the watcher doesn't make a still document due"
kind: bug
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

The human, 2026-10-07 (on zcqv's comments never reaching an agent): "I just want the solution fixed actually."

## What happened

`docs/tickets/open/dalek-slept-in-a-bag-...-zcqv.md` is in `.bridle/review-documents.txt` (added 2026-10-06 ~21:19Z). The human left three comments (c1-c3, 21:19-21:24Z), each marked `[pending ...]`, the newest entry in each thread, committed in 7ebdaed5 and unchanged since. The daemon (pid 15091, up since 22:12Z, build 0.5.0) never sent them: no doc-zcqv agent, nothing from `bridle_daemon::doc_watch` in `.bridle/daemon.log` since 2026-10-05, and more than 2 h past `[review] quiet_minutes` (7).

`bridle review now <path>` at ~00:45Z sent all 3 threads and started doc-zcqv at once. So parsing, naming (wjhp's fix) and sending work; the automatic path (`DocWatcher::tick_at` in `crates/bridle-daemon/src/doc_watch.rs`: observe -> due -> admit -> send) never makes the document due, or the tick never runs.

## Fix

Find why and fix it. Candidates to check first: whether the tick is scheduled at all (wired in `crates/bridle-daemon/src/lib.rs` ~643; daemon.md says every 30 s); whether `observe` resets the quiet clock each tick (e.g. `sweep_marks` rewriting the file, or a comparison that never stabilises); whether a `[pending ...]` mark written by the gateway counts as pending in `due`.

Test: a daemon-level test (not just the core) with a registered document holding a human thread marked `[pending <time>]`, a short quiet period, and the real tick loop: the document's agent gets the batch with no `review now`. The existing tests missed this.

Acceptance: just check passes; the test above fails on today's code and passes after; docs/design/agent-host/daemon.md "Document review" still true (fix it if not). Model: Sonnet.
