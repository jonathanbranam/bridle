+++
id = "br-y7kt"
title = "ticket ready checks the ticket is committed on main, not the project's integration branch"
kind = "bug"
state = "integrated"
created_at = "2026-10-18T23:51:48.693Z"
updated_at = "2026-10-19T11:14:59.495272Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wy7kt",
]
branch = "bridle/wy7kt"
commit = "41a01f173f2a11023ede1d0c7d1a5885cd6c583d"
summary = "ensure_committed (crates/bridle/src/ticket.rs) now takes the branch; ready_cmd passes [branches] integration from Config::load (default main); error names the branch. Test covers dev (on dev ok, only-on-main refused) and main. cli.md and CHANGELOG updated. Other hardcoded main in crates/ are test fixtures or unrelated (doctor.rs already uses the config); none left in this path."
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

### note · agent:wy7kt · 2026-10-19T11:06:56.255Z
built: 6a6da48458dd4f9d81401c4f839fcb3a603d2921 (branch bridle/wy7kt)

### note · agent:wy7kt · 2026-10-19T11:06:56.337Z
done: ticket ready checks the integration branch; just check exit 0, 1888 passed; 6a6da48458dd4f9d81401c4f839fcb3a603d2921

### note · agent:manager-2 · 2026-10-19T11:14:59.495Z
integrated: 41a01f173f2a11023ede1d0c7d1a5885cd6c583d (branch bridle/wy7kt)
