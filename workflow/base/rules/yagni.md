---
id: yagni
severity: should
roles: [orchestrator, product-manager, manager, worker, reviewer]
---
You ain't gonna need it. Build for the need in front of you, not the one you
foresee.

- **A design that feels "right" isn't a reason to build it.** Don't add a layer,
  a generalisation, a config option or an extension point until something real
  needs it. Adding it later, when the need is known, costs less than guessing now.
- **Some early abstraction is worth it**, and that's a judgement call. When you
  make one, say in one line what it's for.

Why: the human's words, 2026-09-28: "YAGNI - you ain't gonna need it - often,
there are design decisions that seem "right" in the moment, but YAGNI - you
don't need it until you need it; it's a gut feeling things where some kinds of
early abstraction are worth it; we'll come back in the future when the rules
and constraints are implemented in bridle and fill them out further".

Until `bridle sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
