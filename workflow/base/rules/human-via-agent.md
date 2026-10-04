---
id: human-via-agent
severity: should
roles: [orchestrator, project-manager, manager, worker, reviewer, document-reviewer, advisor]
---
When you carry the human's words or request (a relay, a "resolve this", a "thanks"), the human
is the author and you are the route: write `human via <agent>`, as in the relay convention
"From the human, via advisor:". When you act on your own judgement, write `<agent>` alone.

- `<agent>` is the agent's name, not its role: `doc-3haz`, `advisor`, `orchestrator`. A name
  identifies one agent you can look up (`bridle agents`, its logs); a role can be reassigned.
  Add the role only if the name doesn't make it clear.
- The human's authority makes the action legitimate; the line is for traceability, as in
  Gmail's From/Sender pair.

Why: the human's words, 2026-10-04: "As a rule, active agents that I talk to can always act on
my behalf ... it would be nice for traceability." Recommendation approved: "human via the
advisor" over "advisor, role authorized by human" (ticket ehv6).
