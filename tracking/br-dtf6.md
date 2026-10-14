+++
id = "br-dtf6"
title = "Per-project focus and budget switches, and one policy for the whole cluster (notes assistant always on)"
kind = "explore"
state = "integrated"
created_at = "2026-10-13T21:38:08.438Z"
updated_at = "2026-10-14T07:40:19.144628Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:dtf6",
]
branch = "bridle/dtf6"
commit = "60cf413dde5da8d63ced13b7902bafba5b0bd1c3"
summary = "Piece 1: governor.rs check_pace sends one note/day to the human and orchestrator when seven_day is over day/7 line + [budget] pace_margin (default 10); warning only. Piece 2: [[budget.schedule]] periods take seven_day/seven_day_opus/seven_day_sonnet threshold tables (config.rs, lower-only under projects); max_workers applies while a period matches by the clock; a once-a-week log warns when no weekly-table period covers the minute before resets_at (log only, doctor not changed). Docs: usage-and-budget.md, roles-and-config.md, CHANGELOG."
ticket = "dtf6"
+++

docs/tickets/open/per-project-focus-and-budget-switches-and-one-policy-for-the-dtf6.md

## Thread

### note · external:advisor/product-manager · 2026-10-14T02:31:26.325Z
The human, 2026-10-13 ~10:30 PM ET (to advisor product-manager), verbatim: "Go ahead with the ticket idea to have a scaling daily limit and a sprint implementation. I think it means the budget blocks need to be able to set all limits including 7 day and opus 7 day. Let's ship that". Scope readied: budget pacing (daily line d/7, warn at line+10) and the end-of-week sprint, with [[budget.schedule]] blocks able to set every limit incl. seven_day and seven_day_opus. The per-project focus/budget switches and cluster policy parts are NOT in this go.

### note · agent:pm-1 · 2026-10-14T03:54:58.439Z
Brief (pm-1). Model: Sonnet. Size: M. Human wants it shipped tonight, before the 7:00 AM reset. SCOPE is the PdM's last comment only: budget pacing (warning) and the end-of-week sprint. NOT in scope: per-project focus/budget switches, the cluster policy, the notes local role, locked-mode advisor change. Stop points: if either piece gets long, land piece 1 and tell the manager; piece 2 can follow as its own ticket.

Piece 1, pace warning (no holds): from the 7-day reset (resets_at, known from statusline / rate_limit_event), day d (1..7, counted from the reset, whole days) has line = d/7 of the week (14%, 28%, ... 100%), granted in full when the day starts. When seven_day utilization exceeds line + margin (config, default 10 points), send ONE message to the orchestrator and the human ("more than 10 points over the pace line"), once per day-and-crossing (do not re-fire every poll; see the 63nr fix for the wake-refire pitfall). Warning only: existing thresholds and holds unchanged.

Piece 2, sprint via schedule: let a [[budget.schedule]] period also set the seven_day AND seven_day_opus thresholds (today it replaces only five_hour; "seven_day is never affected" in usage-and-budget.md), and max_workers while the period matches by the clock. So the human writes one period (name = "sprint", days = ["wed"], start = "00:00", end = "03:00", everything 100). Guard: warn (doctor / log) when a period's day/time does not match the real reset day from resets_at.

Files: crates/bridle-daemon/src/governor.rs, config.rs, crates/bridle-api/src/types.rs (if status shows the line), crates/bridle/src/doctor.rs, crates/bridle-daemon/tests/governor_test.rs; docs/design/usage-and-budget.md, roles-and-config.md, cli.md, CHANGELOG.md. Config change is additive (new optional keys): no migration. Tests: warning fires once when crossing; a sprint period overrides seven_day and seven_day_opus inside its window and not outside. Acceptance: just check. Out of scope: hard caps / wind-down margins on the line, hourly accrual, cluster-shared config.

### note · agent:dtf6 · 2026-10-14T06:51:28.776Z
built: 56c9e01d (branch bridle/dtf6)

### note · agent:dtf6 · 2026-10-14T06:51:28.818Z
done: pace warning + sprint via schedule weekly tables; just check exit 0, 1661 tests passed; 56c9e01d

### note · agent:dtf6 · 2026-10-14T07:10:26.474Z
merged main (808b6f91) into branch; just check exit 0, 1662 passed; tip b0d668e0

### note · agent:manager-2 · 2026-10-14T07:39:32.154Z
integrated: 60cf413dde5da8d63ced13b7902bafba5b0bd1c3 (branch bridle/dtf6)

### note · agent:manager-2 · 2026-10-14T07:39:56.524Z
cleanup: removed agent dtf6, branch bridle/dtf6

### note · external:advisor/product-manager · 2026-10-14T07:40:19.144Z
PdM: the approved scope landed in 60cf413d and is verified: daily pace warning (day/7 line + [budget] pace_margin, default 10; one note a day to the human and the orchestrator) and [[budget.schedule]] periods that set seven_day, seven_day_opus and seven_day_sonnet tables plus max_workers. Ticket stays OPEN for the parts not approved (per-project focus/budget switches, cluster-wide policy), pending the human.
