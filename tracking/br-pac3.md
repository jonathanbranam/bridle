+++
id = "br-pac3"
title = "Turns record their Claude session id and context size (ktgd step 1)"
kind = "feature"
state = "built"
created_at = "2026-10-13T23:05:26.791Z"
updated_at = "2026-10-14T01:07:40.027971Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:pac3",
]
branch = "bridle/pac3"
commit = "36bf8de1"
summary = "Migration SCHEMA_V29 adds nullable turns.session_id and turns.context_tokens (old rows NULL). add_turn_start copies agents.session_id (so a renewed agent's turns split by session); end_turn stores the turn's already-reported context_tokens. No agent is woken. Store test covers two sessions plus context; docs: storage.md, CHANGELOG."
ticket = "pac3"
+++

docs/tickets/open/turns-record-their-claude-session-id-and-context-size-ktgd-s-pac3.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:27.659Z
Brief (pm-1). Model: Sonnet. Size: S. The ask is the spec. Start: the turns ledger writer in crates/bridle-daemon/src/supervisor.rs (~1672, where a turn's cost is recorded; br-9y7v already tracks the session there) and store.rs (`turns` insert, migrations; docs/design/storage.md). One migration adds nullable `session_id` and `context_tokens`; old rows NULL (never zero), safe on every project's database. Context comes from what the turn's result already reports (docs/spikes/01-stream-json-findings.md); never wake an agent. Test with the fake claude: renew once -> turns under two session ids. Update the br-xfy7 series/list queries only if they break. Files also: storage.md, CHANGELOG.md. Out of scope: periods, breakdowns, session start/end (2kam).

### note · agent:pac3 · 2026-10-14T00:23:59.489Z
built: 36bf8de1 (branch bridle/pac3)

### note · agent:pac3 · 2026-10-14T00:23:59.524Z
done: turns.session_id + context_tokens (SCHEMA_V29); just check exit 0, 1649 tests passed; 36bf8de1

### note · agent:pac3 · 2026-10-14T01:07:40.027Z
merged main; just check exit 0, 1651 passed; 7f37b13b
