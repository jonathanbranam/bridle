+++
id = "br-h2p5"
title = "bridle usage --week prints UTC times as Eastern and counts the old week's readings on the first day"
kind = "bug"
state = "built"
created_at = "2026-10-19T23:11:39.329Z"
updated_at = "2026-10-20T05:37:01.851161Z"
created_by = "external:advisor/usage"
watchers = [
    "external:advisor/usage",
    "agent:wh2p5",
]
branch = "bridle/wh2p5"
commit = "c09cca31"
summary = "print_week/windows now format times in periods.timezone (new in_zone helper, unit-tested); days() in usage_periods.rs keeps only seven-day readings whose resets_at is within RESET_TOLERANCE of the week's reset, test for a stale old-week reading. Gateway uses the daemon's periods so it is fixed too. The week integration-test fixture expected UTC 15:00 and now expects 11:00 Eastern."
ticket = "h2p5"
+++

docs/tickets/open/bridle-usage-week-prints-utc-times-as-eastern-and-counts-the-h2p5.md

## Thread

### note · external:advisor/usage · 2026-10-19T23:12:00.177Z
From advisor (usage): filed from the week's usage analysis and readied (kind bug, rule ready-without-asking).

### note · agent:wh2p5 · 2026-10-20T05:37:01.363Z
built: c09cca31 (branch bridle/wh2p5)

### note · agent:wh2p5 · 2026-10-20T05:37:01.851Z
done: usage --week times in schedule tz, stale old-week readings filtered from days; check exit 0, 1938 tests; c09cca31989eae1c41c4cfe267e536d809143e1f
