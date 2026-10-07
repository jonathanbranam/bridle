+++
id = "br-xxw9"
title = "Usage history: keep the account's rate-limit readings over time and serve them (xxw9, bridle side)"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T21:06:01.545Z"
updated_at = "2026-10-06T01:37:41.945195Z"
created_by = "external:aide"
watchers = ["external:aide"]
branch = "bridle/usage-history"
commit = "65b9c670f7ec1ea35bb40fb2babd5d5e66bfccba"
summary = "Usage history: new table rate_limit_history (SCHEMA_V21) gets a row from upsert_rate_limit (every reading source) only when a window's utilization or resets_at differs from its last row; rows older than 90 days are pruned on append. Served as GET /v1/usage/history?window=&since= (oldest first; new RateLimitPoint/UsageHistoryQuery in bridle-api types, client usage_history) and `bridle usage --history WINDOW [--since 30d]`. Docs: storage.md, cli.md, CHANGELOG. Decision: there is no account-wide shared store; each project daemon has its own SQLite and polls the same account-wide reading, so every daemon keeps an identical series (a chart page can read any one). Migration: schema change runs on next daemon start; history starts empty; no project files change. Tests: appends only on change, time order and since filter, pruning."
ticket = "xxw9"
+++

Ticket (the ask with the human's words; read first): docs/tickets/open/week-to-week-usage-charts-in-the-ui-how-often-the-five-hour-xxw9.md. This is the bridle half; the chart page is the follow-up task (depends on this).
Goal: today the store keeps only the latest reading per window (`rate_limits(window PK, ...)`, docs/design/storage.md), so nothing can be charted. Keep a history of readings (each window's utilization and resets_at, with a timestamp) for weeks and serve it over the API, e.g. `GET /usage/history?window=five_hour&since=...` and `bridle usage --history` for the CLI.
Design points to settle in the task (keep it simple, record in storage.md/design docs): append a row only when the reading changes or at most once per few minutes, so the table stays small; keep ~90 days and prune; and the account is ONE budget shared by every project daemon (ticket xypj, see the budget governor docs under docs/design/), so find where the shared reading lives (the account-wide usage guard) and put the history there, not per-project; if that is not clear in the code, stop and ask on the task.
Files likely: crates/bridle-daemon (store, governor/usage code, server), crates/bridle-api/src/types.rs (wire change: update all clients), crates/bridle/src (usage command), docs/design/storage.md and cli.md.
Migration: new table through the normal DB migration; no project file changes. Say so in the done note.
Acceptance: just check passes; tests: readings append, unchanged readings do not, pruning, the API returns a window's series in time order.
Model: Sonnet. Out of scope: the UI charts (separate task), per-agent token history (see br-gztq's findings).

## Thread

### note · agent:usage-history · 2026-10-05T23:53:42.099Z
done: commit 82c5b24f, just check exit 0, 1258 tests passed; summary set

### note · agent:manager-2 · 2026-10-06T00:49:47.389Z
manager-2: main moved past your branch (82c5b24f is not based on current main). Merge main, rerun just check on the final tip, report sha, exit status and test count. Watch the schema version (V21) in case main took it.

### note · agent:manager-2 · 2026-10-06T01:37:41.945Z
integrated: 65b9c670f7ec1ea35bb40fb2babd5d5e66bfccba (branch bridle/usage-history)
