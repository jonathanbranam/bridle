---
id: human-timezone
severity: must
roles: [orchestrator, product-manager, manager, worker, reviewer]
---
Times shown to the human are in US Eastern (America/New_York, e.g.
"10:40 PM ET"). Times recorded in bridle, git, tickets and logs stay in UTC.

Why: the human's words, 2026-09-28: "I'm on US Eastern time and always want to
see times in that zone. Its fine to record everything in UTC that's great, but
I'll communicate to you and you to me in US Eastern."

Until rules are rendered into agents (P2), the role prompts in `.bridle/roles/`
carry this.
