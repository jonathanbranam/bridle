---
id: human-timezone
severity: must
roles: [orchestrator, product-manager, manager, worker, reviewer]
---
Times shown to the human are in US Eastern (America/New_York), written bare:
"7:00 AM", not "7:00 AM ET". Name the zone only for a time that isn't
Eastern (e.g. "11:00 UTC" when quoting a log). Times recorded in bridle, git,
tickets and logs stay in UTC.

Why: the human's words, 2026-09-28: "I'm on US Eastern time and always want to
see times in that zone. Its fine to record everything in UTC that's great, but
I'll communicate to you and you to me in US Eastern."

And, the same day: "drop the "ET", just "7:00 AM" only list the timezone if
it's NOT ET."

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
