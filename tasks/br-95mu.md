+++
id = "br-95mu"
title = "A change spec (proposal and design) reviewed for risk and impact before any worker builds: a real gate, not a convention"
kind = "arch-revision"
state = "pending"
created_at = "2026-10-06T21:40:37.579Z"
updated_at = "2026-10-11T04:07:11.090952Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "95mu"
+++

docs/tickets/open/a-change-spec-proposal-and-design-reviewed-for-risk-and-impa-95mu.md

## Thread

### note · external:orchestrator · 2026-10-06T22:18:59.271Z
The human, 2026-10-06 ~6:30 PM ET: adding a new config section before the daemon is upgraded can break it ('that happened before to me with config.toml'), cf. jmpf, where a [gateway] section stopped every daemon. config.toml's structs use deny_unknown_fields (crates/bridle-daemon/src/config.rs, 6 places; also machines.rs and bridle-mail), so an older binary rejects a section a newer one adds. credentials.toml isn't strict. The change-spec review must cover config and credentials compatibility: old binaries tolerate new sections, or the rollout order (upgrade first, then edit config) is stated and checked.

### note · external:advisor/product-manager · 2026-10-09T11:04:39.424Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T04:07:11.090Z
advisor (product-manager): moved out of epic everything-is-a-ticket into reviews-enforced, renamed 'Gates: design before build, review before landing' (the human, 2026-10-11 ~1:30 AM ET: 'yes, agreed move it'). Not on the path to everything-is-a-ticket's done-when; it pairs with v2va (gate at the start vs the end). 22ab step 6 notes it.
