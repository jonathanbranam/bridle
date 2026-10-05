+++
id = "br-xxw9"
title = "Usage history: keep the account's rate-limit readings over time and serve them (xxw9, bridle side)"
kind = "feature"
state = "planned"
created_at = "2026-10-05T21:06:01.545Z"
updated_at = "2026-10-05T21:06:46.912037Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

original id: xxw9
Ticket (the ask with the human's words; read first): docs/tickets/open/week-to-week-usage-charts-in-the-ui-how-often-the-five-hour-xxw9.md. This is the bridle half; the chart page is the follow-up task (depends on this).
Goal: today the store keeps only the latest reading per window (`rate_limits(window PK, ...)`, docs/design/storage.md), so nothing can be charted. Keep a history of readings (each window's utilization and resets_at, with a timestamp) for weeks and serve it over the API, e.g. `GET /usage/history?window=five_hour&since=...` and `bridle usage --history` for the CLI.
Design points to settle in the task (keep it simple, record in storage.md/design docs): append a row only when the reading changes or at most once per few minutes, so the table stays small; keep ~90 days and prune; and the account is ONE budget shared by every project daemon (ticket xypj, see the budget governor docs under docs/design/), so find where the shared reading lives (the account-wide usage guard) and put the history there, not per-project; if that is not clear in the code, stop and ask on the task.
Files likely: crates/bridle-daemon (store, governor/usage code, server), crates/bridle-api/src/types.rs (wire change: update all clients), crates/bridle/src (usage command), docs/design/storage.md and cli.md.
Migration: new table through the normal DB migration; no project file changes. Say so in the done note.
Acceptance: just check passes; tests: readings append, unchanged readings do not, pruning, the API returns a window's series in time order.
Model: Sonnet. Out of scope: the UI charts (separate task), per-agent token history (see br-gztq's findings).
