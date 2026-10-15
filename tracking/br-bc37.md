+++
id = "br-bc37"
title = "Mail digest: a failed digest raises an event, and one unknown ticket kind/state no longer empties the task section (follow-up to tcae)"
kind = "fix"
state = "open"
created_at = "2026-10-15T20:52:58.094Z"
updated_at = "2026-10-15T21:00:50.194372Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
ticket = "bc37"
+++

docs/tickets/open/mail-digest-a-failed-digest-raises-an-event-and-one-unknown-bc37.md

## Thread

### note · agent:pm-1 · 2026-10-15T20:53:09.244Z
Brief (pm-1). Model: Sonnet. Size: S/M. Parent br-tcae stays open until the PdM resolves it. Two parts. (1) A failed digest must be seen: today it is only tracing::warn (crates/bridle-mail/src/bridge.rs ~244-255, failures.rs). Raise it the way the bridge already surfaces other failures to the human (look at failures.rs for the existing mechanism: an event or a message to the orchestrator/human); if none fits, send one message per failure kind, rate-limited to once per day so a stuck digest does not spam. (2) Tolerance: the wire enums for ticket kind/state (bridle-api types.rs) lack serde(other), so one unknown kind or state makes the whole task section of the digest fail to parse and come out empty. Fix so an unknown value deserializes to an Unknown variant (or the bridge skips just that item, with a warn naming it) and the rest of the digest renders. Careful: types.rs is the wire contract; adding an Unknown variant must not change serialized output for known values. Tests: a digest input with one unknown kind and one unknown state still lists the other tasks; a failed digest raises the event. Docs: mail bridge design doc, CHANGELOG. Acceptance: just check. Out of scope: the stale-binary fix (landed in tcae). Touches the same crate as br-m38a/br-6gkv/br-257v (bridge.rs vs parse.rs): no shared file expected; if one appears, run after them.
