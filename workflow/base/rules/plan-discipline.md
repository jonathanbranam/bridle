---
id: plan-discipline
severity: should
roles: [orchestrator, product-manager, manager, worker, reviewer]
---
A plan is complete when it says what it touches, what it defers and why, what
it rejected, and how to check it.

- **Name every file it may touch.** The impact list is the boundary; work
  outside it needs the plan changed first.
- **A deferral is "not yet", with a reason.** Never a bare "out of scope".
- **Design records rejected alternatives**, so the next reader does not
  re-propose them.
- **Each task states how to verify it.**

Why: data-contracts' `config.yaml` rules. They map onto `bridle-plan` and the
impact declaration.

Until `bridle sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
