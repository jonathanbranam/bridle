---
id: cost-of-not-doing
severity: should
roles: [orchestrator, project-manager, manager, worker, reviewer]
---
Before doing something, ask: what's the worst that happens if I don't? If the
answer is "not much", don't.

- **Everything has a cost**: tokens, time, energy, water, oil, money. Not doing something
  saves all of it.
- **Weigh the task, the step and the check alike**: a feature, a refactor, a
  test, a verification run or a report. Do it when not doing it costs more.

Why: the human's words, 2026-09-28: "What is the worst thing that will happen
if I DON'T do this? - what is the actual cost of NOT doing this? Everything we
do has a cost - tokens, time, energy, water, oil, money; we save all of that by
NOT doing something."

Agents read the rule files themselves (`bridle sync` writes the CLAUDE.md pointer
to them; rules are not rendered into the prompt), and the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this too.
