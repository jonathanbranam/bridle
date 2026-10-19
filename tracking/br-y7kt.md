+++
id = "br-y7kt"
title = "ticket ready checks the ticket is committed on main, not the project's integration branch"
kind = "bug"
state = "planned"
created_at = "2026-10-18T23:51:48.693Z"
updated_at = "2026-10-19T00:15:11.934758Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "y7kt"
+++

docs/tickets/open/ticket-ready-checks-the-ticket-is-committed-on-main-not-the-y7kt.md

## Thread

### note · external:aide · 2026-10-18T23:51:56.760Z
Readied by aide: a bug with one clear fix (rule ready-without-asking).

### note · agent:pm-1 · 2026-10-18T23:52:31.950Z
Brief (pm-1). Model: Sonnet. Priority high: it blocks readying any track-web ticket (integration branch is dev), so track-web work cannot start. Do the ask: `ensure_committed` (crates/bridle/src/ticket.rs ~504-517) checks the project's integration branch (`[branches] integration`, default main) via the existing config reader; fix the error text; grep crates/ for other hardcoded `main:`/"main" where the integration branch is meant and fix those found in the same code path (list any you leave on this thread). Test: a project with integration = "dev" readies a ticket committed on dev and refuses one only on main; default project unchanged. Docs: CHANGELOG, and docs/design/cli.md if it says "main". Acceptance: just check. Out of scope: the migrations branch guard (br-rdag).

### note · external:aide · 2026-10-18T23:53:22.236Z
The human, 2026-10-18 (via track-web's aide, m-17010), verbatim: "this is the situation exactly. The ticket change with main vs. dev." Confirms this is the dev-vs-main trouble the human heard about; no migration misbehaved beyond it.
