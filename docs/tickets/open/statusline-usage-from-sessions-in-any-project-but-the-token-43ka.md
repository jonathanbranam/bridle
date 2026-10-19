---
id: br-43ka
title: Statusline usage from sessions in any project but the token's is dropped silently (orchestrator in dalek-home, aides in other projects)
kind: bug
created: 2026-10-19
created_by: external:advisor/usage
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-368g, br-a65d]
---

## The ask

The human, 2026-10-19 (to advisor usage, while analysing the week's usage): "Please note any issues like this and investigate and file bugs to address them".

`bridle statusline` drops the usage of every interactive session whose cwd is in a project other than the one that minted `~/.bridle/statusline.token`. It drops them silently, so `bridle usage` and its share-of-week numbers undercount. Found by advisor usage, 2026-10-19 ~7:15 PM ET.

What was seen on dalek, week of 2026-10-14:

- `external:orchestrator` reported to bridle's daemon until Sun 2026-10-18 09:50 ET, when it moved to dalek-home (`orch-dalek-home`, cwd `/Volumes/Data/work/dalek-home-work/dalek-home`). It has posted nothing since: dalek-home's `interactive_usage` has no rows. It ran at about $14/day (Opus) before the move, so roughly 1.5 days (about 2 of the week's 60 points) are missing.
- `aide-track-web` (running since 2026-10-18 19:49 ET) has posted nothing: track-web's `interactive_usage` has no rows.
- The day-by-day points per dollar rose on 10-19 (0.142 against about 0.12 on other days), which fits spend the ledger doesn't see.

Cause: `statusline.token` is one per-user file, minted with `bridle --project <name> token create statusline` (docs/design/cli.md, "One-time setup"). So it exists as a principal only on that project's daemon. On dalek, only bridle's `principals` table has `external:statusline`; dalek-home, track-web and bridle-ui have none. `post_statusline_report_via` (crates/bridle/src/commands/orchestrator.rs) resolves the daemon from the session's cwd, sends this token, gets refused, and logs the failure only at `tracing::debug`. Nothing tells the human or the orchestrator. The fallback daemon path is used only when no project daemon is found, not when one rejects the token.

The ask: every interactive session's usage is recorded wherever it runs on the machine, and a refused or failed post is visible somewhere (for example, a count or a warning in `bridle usage`, or a daemon incident) instead of only a debug log. Possible fixes, for whoever builds it: make the statusline credential machine-wide (every local daemon accepts it); have `bridle project add`/`setup` mint it on each daemon; or have the statusline fall back to the minting daemon, with the session's real project on the row. Update docs/design/cli.md and docs/design/usage-and-budget.md ("External sessions") to match.

Not in scope: the desktop app and claude.ai, which stay uncounted, and the human's own sessions (ticket 368g).
