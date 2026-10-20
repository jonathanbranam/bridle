---
id: br-3qha
title: What should each ticket watcher be woken for? Notices are 69% of the messages that wake the Opus sessions
kind: question
created: 2026-10-20
created_by: external:advisor/usage
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-xxxq, br-9nrt]
---

## The ask

The human, 2026-10-19 ~8:00 PM ET (to advisor usage), on the advisor's suggestion to batch the notices that wake interactive Opus sessions:

> I'm not sure about batching the notifications exactly. We could batch them up over a minute or so but would that really help much?
>
> Maybe instead we should refine the way that ticket watchers/creators subscribe to notifications. It's something to consider for sure.

What the data says (advisor usage, bridle's daemon, week of 2026-10-14 to 10-19):

- 1,520 messages went to interactive sessions (orchestrator, aide, advisors). 1,047 of them (69%) were ticket-watch notices, `<id> (<title>): <change> by <who>`, and 507 of those were "comment by ..." notices.
- Every notice ends the recipient's `bridle agent wake`, so it costs an Opus turn that re-reads the session's context. The median statusline report is about $0.035, and interactive sessions were about 42 of the week's 60 points (`scripts/usage-report.py`).
- Batching to 60 s would cut the 1,047 notices to at most 565 wakes, and 10 minutes to 446. That is an upper bound: a waiter already gets every message queued while it was busy. So batching saves something, but it doesn't change what a watcher is woken for.
- Example: advisor usage filed four bug tickets, and pm-1's four "open -> planned" notices woke it twice. The filer had nothing to do with either change.

The question for the human: what should each kind of watcher be woken for? Some directions to consider, not a design:

- Watch levels per watcher: "everything", "decisions" (questions to me, answers, holds, done/resolved, sent back), "none, just list it". A creator who only filed the ticket (an advisor, the aide) defaults to "decisions"; a claimer or manager gets more.
- Notices that don't wake: they are queued, and the next real wake (or `bridle inbox`) shows them as a digest.
- Per-role defaults in config, so the orchestrator's and advisors' Opus sessions are woken less than the Sonnet managers.

Builds on [[task-watchers-a-creator-field-a-watchers-list-and-wakes-that-xxxq|xxxq]] (watchers) and [[what-it-means-for-the-human-to-watch-a-task-9nrt|9nrt]] (the human watching a task).
