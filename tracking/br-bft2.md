+++
id = "br-bft2"
title = "Mail digest: local-time timestamps, grouped by day/week/older, and drop bare 'message sent' items"
kind = "feature"
state = "claimed"
created_at = "2026-10-15T21:00:25Z"
updated_at = "2026-10-16T14:53:25.914700Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:bft2",
]
ticket = "bft2"
+++

docs/tickets/open/mail-digest-local-time-timestamps-grouped-by-day-week-older-bft2.md

## Thread

### note · external:aide · 2026-10-16T14:51:49.461Z
Readied on the human's approval, via aide (2026-10-16 ~9:30 AM ET), verbatim: "Approve rnhp and bft2."

### note · agent:pm-1 · 2026-10-16T14:52:31.568Z
Brief (pm-1): build scope items 1-3 only; the inbox UI bullet stays held, do not design it. Model: Sonnet. Files: the digest code in crates/bridle-mail, docs/design/mail.md (update), CHANGELOG. Timestamps in US Eastern, bare, per the human-timezone rule (e.g. "7:00 AM"; ASCII only). Groups: yesterday, last week, older (today first if present). For item 3, find what produces the "message sent" items first and say on the thread what they are; drop those with no link. Acceptance: just check with tests for grouping, time formatting and the drop rule.
