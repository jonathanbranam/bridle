---
id: out-of-scope
severity: must
roles: [worker, reviewer]
---
File out-of-scope findings; do not fix them. Never weaken a test to make it
pass.

- **Something else is wrong?** Note it in your report or file a ticket. Fixing
  it widens the change and its review.
- **A failing test is information.** Fix the code, or if the test is wrong say
  so and why. Do not loosen, skip or delete it to get green.

Why: data-contracts' `config.yaml` `operations.apply`. `bridle-worker` already
says to note rather than fix; this makes it a rule, and adds the test half.

Until `bridle sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
