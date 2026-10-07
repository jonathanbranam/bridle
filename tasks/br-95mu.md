+++
id = "br-95mu"
title = "A change spec (proposal and design) reviewed for risk and impact before any worker builds: a real gate, not a convention"
kind = "arch-revision"
state = "pending"
created_at = "2026-10-06T21:40:37.579Z"
updated_at = "2026-10-06T22:18:59.271758Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "95mu"
+++

docs/tickets/open/a-change-spec-proposal-and-design-reviewed-for-risk-and-impa-95mu.md

## Thread

### note · external:orchestrator · 2026-10-06T22:18:59.271Z
The human, 2026-10-06 ~6:30 PM ET: adding a new config section before the daemon is upgraded can break it ('that happened before to me with config.toml'), cf. jmpf, where a [gateway] section stopped every daemon. config.toml's structs use deny_unknown_fields (crates/bridle-daemon/src/config.rs, 6 places; also machines.rs and bridle-mail), so an older binary rejects a section a newer one adds. credentials.toml isn't strict. The change-spec review must cover config and credentials compatibility: old binaries tolerate new sections, or the rollout order (upgrade first, then edit config) is stated and checked.
