+++
id = "br-82qp"
title = "schedule list hides schedules that target the caller"
kind = "bug"
state = "integrated"
created_at = "2026-10-14T17:02:01.140Z"
updated_at = "2026-10-14T23:14:52.544264Z"
created_by = "external:orchestrator@vesta"
watchers = [
    "external:orchestrator@vesta",
    "agent:82qp",
]
branch = "bridle/82qp"
commit = "a6d3ee2ec73fc1da0843354da7b42ddcb20f4b9e"
summary = "schedule list now returns schedules the caller created or that target the caller (store query + handler), with additive Schedule.relation (created|target) for non-human callers; CLI table adds '(set by X)' for target ones. rm unchanged. Tests in schedule_test.rs; docs cli.md, api.md, CHANGELOG."
ticket = "82qp"
+++

## The ask

Reported by the notes aide (vesta), relayed by vesta's orchestrator. `bridle schedule list` run as external:aide on vesta's notes daemon returns [] although sc-9rnk (created by the human, cron `0 2 * * *`) targets external:aide; the human sees it with their token. The list filters by creator, so an agent cannot see the schedules that wake it. The human, 2026-10-14 1:01 PM ET: "you can't see this yourself?"

`schedule list` also shows schedules whose target is the caller, marked as such. Removing stays creator-only (or the human).

Verify: the human creates a schedule targeting external:aide; `bridle schedule list` as aide shows it.

## Thread

### note · external:orchestrator@vesta · 2026-10-14T17:02:01.171Z
submitted by external:orchestrator@vesta

### note · agent:pm-1 · 2026-10-14T17:02:15.522Z
Accepted by pm-1. Brief. Model: Haiku or Sonnet (small, mechanical). Size: XS-S. Find the list handler: git grep -n "schedule" crates/bridle-daemon/src/server.rs and the store query that filters by creator. Change: list returns schedules created by the caller OR targeting the caller, with a marker field (e.g. `relation: created|target`, additive in crates/bridle-api/src/types.rs) and the CLI table showing it. rm unchanged (creator or human). Tests: caller sees a schedule created by another principal that targets it, marked; cannot remove it. Docs: docs/design/cli.md (schedule), api.md, CHANGELOG.md. No migration. Acceptance: just check. Out of scope: letting a target remove or edit.

### note · agent:pm-1 · 2026-10-14T19:38:25.208Z
pm-1: merged br-ggd7 (same bug from the vesta aide). One addition to the brief: the list also shows who set each schedule (creator), not only the created/target marker.

### note · agent:82qp · 2026-10-14T22:58:28.622Z
done: schedule list shows created + targeting-caller schedules with relation marker; just check exit 0, 1684 tests passed; 9b2b1d48

### note · agent:manager-2 · 2026-10-14T22:58:33.997Z
built: 9b2b1d48 (branch bridle/82qp)

### note · agent:manager-2 · 2026-10-14T23:14:36.068Z
integrated: a6d3ee2ec73fc1da0843354da7b42ddcb20f4b9e (branch bridle/82qp)

### note · agent:manager-2 · 2026-10-14T23:14:52.544Z
cleanup: removed agent 82qp, branch bridle/82qp
