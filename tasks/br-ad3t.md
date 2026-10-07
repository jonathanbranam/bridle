+++
id = "br-ad3t"
title = "Document review never sends the human's comments on its own: the watcher doesn't make a still document due"
kind = "bug"
state = "integrated"
created_at = "2026-10-07T00:43:14.412Z"
updated_at = "2026-10-07T02:03:40.583235Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/ad3t-doc-review"
commit = "e4a617eaf2bc9612ad5e66ee2760f98c13e219de"
summary = "Cause: DocWatcher::send re-reads the file before sending and compares the batch it observed against the current pending text via unid(); unid only stripped the thread ID from the first block's prefix of the joined batch. A document whose threads already had IDs (c1.., as the gateway writes them) never matched, so the tick returned Ok(0) every time and only `review now` (no expect) worked. Fix: compare per block (unid_batch). Added Overrides.doc_watch_interval (default 30 s) so a daemon-level test can run the real tick loop; new tests/review_tick_test.rs (fails on old code, 60 s timeout; passes now) plus a unit test. Debug tracing lines for held back / due / slot-waiting / changed / nothing pending. daemon.md and CHANGELOG updated."
+++

original id: ad3t
Ticket (the human's words, what happened, candidates, the required test; read all of it): docs/tickets/open/document-review-never-sends-the-human-s-comments-on-its-own-ad3t.md . Approved by the human 2026-10-07: "I just want the solution fixed actually."
Bug: the automatic document-review path never sends a still document's pending human comments; `bridle review now <path>` works. Find the cause and fix it. Start in crates/bridle-daemon/src/doc_watch.rs (`DocWatcher::tick_at`: observe -> due -> admit -> send) and the tick's wiring in crates/bridle-daemon/src/lib.rs (~643; every 30 s per docs/design/agent-host/daemon.md "Document review"). Check, in this order: (1) the tick is actually scheduled and runs (log or test it); (2) `observe` resets the quiet clock each tick (a comparison that never stabilises, or `sweep_marks` rewriting the file so its mtime/hash changes every tick); (3) a `[pending <time>]` mark written by the gateway is counted as pending by `due`; (4) `[review] quiet_minutes` (default 7) is read from the right config level.
Required test (the gap): a daemon-level test, not only the core: a registered document holding a human thread marked `[pending <time>]`, a short quiet period, and the REAL tick loop (fake clock only if the loop supports it); assert the document's agent is started and gets the batch with no `review now`. It must fail on today's code and pass after the fix. Use the fake claude; no live tests.
Docs: docs/design/agent-host/daemon.md "Document review" must stay true; fix it if the cause shows it wrong. CHANGELOG entry (read with a limit). Add a debug-level tracing line when a document becomes due or is held back, and why, so the next occurrence is diagnosable.
Acceptance: just check passes; the test above. Migration: none. Model: Sonnet. Out of scope: gateway UI changes, review content/format changes.

## Thread

### note · external:orchestrator · 2026-10-07T00:43:14.578Z
From orchestrator: approved by the human, 2026-10-07: 'I just want the solution fixed actually.'

### note · agent:ad3t-doc-review · 2026-10-07T01:59:57.460Z
done: still-document auto send fixed (batch compare ignored IDs wrongly); just check exit 0, 1265 tests; 05e729eb

### note · agent:ad3t-doc-review · 2026-10-07T01:59:59.138Z
done: root cause = the pre-send 'text unchanged' check compared batches with thread IDs vs without (unid only handled the first block), so numbered docs never sent. Fixed; test in tests/review_tick_test.rs; docs and CHANGELOG updated. just check exit 0, 1265 tests, after merging main; commit 05e729eb.

### note · agent:manager-2 · 2026-10-07T02:00:02.748Z
manager-2: main moved (br-m63h landed 96c8f7ef). Merge main into your branch, rerun just check once, update the done note with the new sha and test count.

### note · agent:ad3t-doc-review · 2026-10-07T02:03:26.890Z
updated: merged main again; just check exit 0, 1266 tests; commit fee3a2cd (supersedes 05e729eb).

### note · agent:ad3t-doc-review · 2026-10-07T02:03:31.969Z
done: merged main, just check exit 0, 1266 tests; fee3a2cd

### note · agent:manager-2 · 2026-10-07T02:03:40.583Z
integrated: e4a617eaf2bc9612ad5e66ee2760f98c13e219de (branch bridle/ad3t-doc-review)
