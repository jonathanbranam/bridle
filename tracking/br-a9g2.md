+++
id = "br-a9g2"
title = "agent spawn --allow-tool silently does nothing for a tool outside the role's --tools set (WebSearch on a worker)"
kind = "fix"
state = "claimed"
created_at = "2026-10-07T00:11:13.981Z"
updated_at = "2026-10-17T23:54:00.582287Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
    "agent:wa9g2",
]
ticket = "a9g2"
+++

docs/tickets/open/agent-spawn-allow-tool-silently-does-nothing-for-a-tool-outs-a9g2.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T11:04:39.766Z
watching the task

### note · external:advisor/product-manager · 2026-10-11T07:22:56.771Z
kind: bug -> fix

### note · external:advisor/product-manager · 2026-10-17T15:26:07.290Z
Readied by advisor (product-manager) 2026-10-17 ~11:45 AM ET under the human's rule (br-mhjn): bugs, fixes and chores go through without asking. Checked still real on main.

### note · external:orchestrator · 2026-10-17T23:54:00.582Z
orchestrator: take the refusal option. Refuse the spawn with '<tool> isn't in the <role> role's tool set; use the researcher role'. Adding the tool to --tools would widen a role's tool set from the spawn line, and the human's standing decision (bridle role file) holds anything that widens an agent's permissions for them. If the refusal turns out too strict, file that as a separate ticket for the human.
