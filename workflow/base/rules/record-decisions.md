---
id: record-decisions
severity: should
roles: [orchestrator, project-manager, manager, worker]
---
Record a decision where it outlives the change that produced it.

- **Not in the ticket, the PR or the chat.** Those are scaffolding. Put it in
  the design doc, a spec, a rule, or a question ticket's resolution.
- **Same commit as the change**, when the change is what decided it.

Why: data-contracts' `driver-guide.md` §2 and `README.md` rule 5. Tickets are
archived; the record of why must not go with them. Git is the source of truth.

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
