+++
id = "br-tqtp"
title = "Document review never sends the human's comments on its own: the watcher doesn't make a still document due"
kind = "bug"
state = "dropped"
created_at = "2026-10-07T00:43:14.523Z"
updated_at = "2026-10-07T00:43:53.172695Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "ad3t"
+++

Ticket (the human's words, what happened, candidates, the required test; read all of it): docs/tickets/open/document-review-never-sends-the-human-s-comments-on-its-own-ad3t.md . Approved by the human 2026-10-07: "I just want the solution fixed actually."
Bug: the automatic document-review path never sends a still document's pending human comments; `bridle review now <path>` works. Find the cause and fix it. Start in crates/bridle-daemon/src/doc_watch.rs (`DocWatcher::tick_at`: observe -> due -> admit -> send) and the tick's wiring in crates/bridle-daemon/src/lib.rs (~643; every 30 s per docs/design/agent-host/daemon.md "Document review"). Check, in this order: (1) the tick is actually scheduled and runs (log or test it); (2) `observe` resets the quiet clock each tick (a comparison that never stabilises, or `sweep_marks` rewriting the file so its mtime/hash changes every tick); (3) a `[pending <time>]` mark written by the gateway is counted as pending by `due`; (4) `[review] quiet_minutes` (default 7) is read from the right config level.
Required test (the gap): a daemon-level test, not only the core: a registered document holding a human thread marked `[pending <time>]`, a short quiet period, and the REAL tick loop (fake clock only if the loop supports it); assert the document's agent is started and gets the batch with no `review now`. It must fail on today's code and pass after the fix. Use the fake claude; no live tests.
Docs: docs/design/agent-host/daemon.md "Document review" must stay true; fix it if the cause shows it wrong. CHANGELOG entry (read with a limit). Add a log line (tracing) when a document becomes due or is held back and why, at debug level, so the next occurrence is diagnosable.
Acceptance: just check passes; the test above. Migration: none. Model: Sonnet. Out of scope: gateway UI changes, review content/format changes.

## Thread

### note · agent:pm-1 · 2026-10-07T00:43:25.089Z
pm-1: filed from ticket ad3t (the orchestrator's br-ad3t id did not exist). Orchestrator or human: please run 'bridle task ready br-tqtp'; I then plan it and queue it after br-x56y and br-76aq.

### note · agent:pm-1 · 2026-10-07T00:43:53.172Z
dropped: duplicate of br-ad3t, filed at the same instant by mistake
