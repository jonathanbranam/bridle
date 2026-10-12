+++
id = "br-9y7v"
title = "Usage shows negative costs for long-running agents: the cost delta goes wrong when the agent's claude session is replaced"
kind = "bug"
state = "integrated"
created_at = "2026-10-12T22:39:44.235Z"
updated_at = "2026-10-12T23:27:25.561792Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:w9y7v",
]
branch = "bridle/w9y7v"
commit = "dab3592f647e9a71b0aa39ab1679c8313425f847"
summary = "Cause confirmed: after a renew (or a resume that falls back to a fresh session) the new claude session's total_cost_usd restarts at 0, but supervisor.rs seeded last_cumulative from the agent's lifetime cost_usd_total, so the first turn's delta was (small - large) = negative. Fix: new agents.session_cost_usd (migration 27) holds the session's own last total, reset to 0 by set_agent_session; the runtime baseline reads it, each result persists it, and a delta is clamped at >= 0. Migration 27 zeroes negative turns.cost_total (true value not recoverable) and re-sums cost_usd_total for agents that had one; existing agents' baseline starts at their lifetime total. Rows repaired: in the live bridle.db today (read-only count before migrating) 89 negative turns across 25 agents, sum -354.07 USD; those 89 rows become 0 and the 25 agents' totals are re-summed. Test: renew_keeps_turn_costs_non_negative_and_the_total_summing (fake-claude gained FAKE_CLAUDE_COST_STEP) and v27_migration_repairs_negative_turn_costs. Docs: usage-and-budget.md, storage.md, CHANGELOG. Not done: external sessions (hd28)."
ticket = "9y7v"
+++

docs/tickets/open/usage-shows-negative-costs-for-long-running-agents-the-cost-9y7v.md

## Thread

### note · agent:pm-1 · 2026-10-12T22:40:29.715Z
Brief (pm-1). Model: Sonnet. Size: S-M. Front of the queue (the human is near the weekly limit and wants trustworthy usage numbers). The ask above is the spec; confirm the cause first at crates/bridle-daemon/src/supervisor.rs (~1672, `last_cumulative`) and say in a comment whether it is the session change. Fix: reset the baseline whenever the claude session id changes; never store a negative delta. Repair existing rows through a normal DB migration (docs/design/storage.md; state the number of rows changed in the summary; the migration must be safe to run on every project's database). Test with the fake claude (crates/bridle-claude/tests/fake-claude.py): a renew mid-run keeps costs non-negative and summing correctly. Files also: docs/design/usage-and-budget.md if it describes the delta, CHANGELOG.md. Out of scope: external sessions (br-hd28).

### note · agent:w9y7v · 2026-10-12T23:26:45.351Z
built: e3e9bff1 (branch bridle/w9y7v)

### note · agent:w9y7v · 2026-10-12T23:26:45.394Z
done: per-session cost baseline + migration 27 (89 negative turns repaired in live db, 25 agents); just check exit 0, 1625 tests, sha e3e9bff1 (main merged)

### note · agent:manager-2 · 2026-10-12T23:27:07.070Z
integrated: dab3592f647e9a71b0aa39ab1679c8313425f847 (branch bridle/w9y7v)

### note · agent:manager-2 · 2026-10-12T23:27:25.561Z
cleanup: removed agent w9y7v, branch bridle/w9y7v
