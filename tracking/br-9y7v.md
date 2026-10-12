+++
id = "br-9y7v"
title = "Usage shows negative costs for long-running agents: the cost delta goes wrong when the agent's claude session is replaced"
kind = "bug"
state = "planned"
created_at = "2026-10-12T22:39:44.235Z"
updated_at = "2026-10-12T22:40:34.477495Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "9y7v"
+++

docs/tickets/open/usage-shows-negative-costs-for-long-running-agents-the-cost-9y7v.md

## Thread

### note · agent:pm-1 · 2026-10-12T22:40:29.715Z
Brief (pm-1). Model: Sonnet. Size: S-M. Front of the queue (the human is near the weekly limit and wants trustworthy usage numbers). The ask above is the spec; confirm the cause first at crates/bridle-daemon/src/supervisor.rs (~1672, `last_cumulative`) and say in a comment whether it is the session change. Fix: reset the baseline whenever the claude session id changes; never store a negative delta. Repair existing rows through a normal DB migration (docs/design/storage.md; state the number of rows changed in the summary; the migration must be safe to run on every project's database). Test with the fake claude (crates/bridle-claude/tests/fake-claude.py): a renew mid-run keeps costs non-negative and summing correctly. Files also: docs/design/usage-and-budget.md if it describes the delta, CHANGELOG.md. Out of scope: external sessions (br-hd28).
