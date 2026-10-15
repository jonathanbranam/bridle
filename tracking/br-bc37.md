+++
id = "br-bc37"
title = "Mail digest: a failed digest raises an event, and one unknown ticket kind/state no longer empties the task section (follow-up to tcae)"
kind = "fix"
state = "integrated"
created_at = "2026-10-15T20:52:58.094Z"
updated_at = "2026-10-15T22:30:23.450352Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:bc37",
]
branch = "bridle/bc37"
commit = "2ccea1ce79e36db552cdeabe214e446e9dff563d"
summary = "Mail digest now tolerates unknown ticket kinds/states and reports a failed digest. ClientSink::tasks decodes tickets one by one (bridge.rs decode_tickets; Client::list_tasks_raw added), skipping an unreadable one with a warn naming it; chosen over serde(other) Unknown variants because TicketKind/TicketState are matched exhaustively across the workspace and the wire contract stays untouched. A digest send failure sends a note to human from external:mail, at most once a day (Bridge.digest_failure_told), then still returns the error so it retries. bridle-mail serde_json moved to regular deps. Tests in outbound_test.rs; docs/design/mail.md and CHANGELOG updated."
ticket = "bc37"
+++

docs/tickets/open/mail-digest-a-failed-digest-raises-an-event-and-one-unknown-bc37.md

## Thread

### note · agent:pm-1 · 2026-10-15T20:53:09.244Z
Brief (pm-1). Model: Sonnet. Size: S/M. Parent br-tcae stays open until the PdM resolves it. Two parts. (1) A failed digest must be seen: today it is only tracing::warn (crates/bridle-mail/src/bridge.rs ~244-255, failures.rs). Raise it the way the bridge already surfaces other failures to the human (look at failures.rs for the existing mechanism: an event or a message to the orchestrator/human); if none fits, send one message per failure kind, rate-limited to once per day so a stuck digest does not spam. (2) Tolerance: the wire enums for ticket kind/state (bridle-api types.rs) lack serde(other), so one unknown kind or state makes the whole task section of the digest fail to parse and come out empty. Fix so an unknown value deserializes to an Unknown variant (or the bridge skips just that item, with a warn naming it) and the rest of the digest renders. Careful: types.rs is the wire contract; adding an Unknown variant must not change serialized output for known values. Tests: a digest input with one unknown kind and one unknown state still lists the other tasks; a failed digest raises the event. Docs: mail bridge design doc, CHANGELOG. Acceptance: just check. Out of scope: the stale-binary fix (landed in tcae). Touches the same crate as br-m38a/br-6gkv/br-257v (bridge.rs vs parse.rs): no shared file expected; if one appears, run after them.

### note · agent:bc37 · 2026-10-15T22:11:55.640Z
done: mail digest skips tickets of unknown kind/state (per-item decode) and tells the human once a day when a digest can't be sent; just check exit 0, 1714 tests passed (band 1712); check ran on the branch before a final clean merge of main (only a new ticket doc came in); 10d825c6

### note · agent:manager-2 · 2026-10-15T22:12:01.376Z
built: 10d825c6 (branch bridle/bc37)

### note · agent:manager-2 · 2026-10-15T22:30:20.827Z
integrated: 2ccea1ce79e36db552cdeabe214e446e9dff563d (branch bridle/bc37)

### note · agent:manager-2 · 2026-10-15T22:30:23.450Z
cleanup: removed nothing; kept agent bc37 (background job pid 31189)
