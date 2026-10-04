---
id: work-flow
severity: must
roles: [orchestrator, product-manager, manager, worker, reviewer]
---
Filing is not scheduling, and a report is a claim.

- **Filing schedules nothing.** Anyone may file a ticket or a finding; that
  commits nobody to doing it. Only the human or a manager decides what is
  worked on.
- **Only the human's (or manager's) merge accepts work.** A worker finishing
  is not the work landing.
- **The plan comes before code.** Implementation starts from an approved plan.
- **A report is a claim, so verify it.** Whoever receives "done" checks the
  result themselves before relying on it.

Why: data-contracts' `workflow-instructions/README.md` rules 1–5. Bridle
already enforces the plan-before-code and filing parts by construction (task
states and the plan gate); this rule states the invariant they serve, and
covers the parts they do not: acceptance and verification.

Agents read the rule files themselves (`bridle sync` writes the CLAUDE.md pointer
to them; rules are not rendered into the prompt), and the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this too.
