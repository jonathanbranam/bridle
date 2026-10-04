---
id: doc-links
severity: should
roles: [orchestrator, product-manager, manager, worker, reviewer]
---
Link between docs from the repo root, in wiki form, with no leading `/` and as
few `../` as possible.

- **Root-relative**: write `docs/design/cli.md`, not `/docs/design/cli.md` or
  `../../design/cli.md`. A link then survives the doc moving.
- **Wiki form preferred**: `[[docs/design/cli]]` over a long relative path
  where the project's docs support it.

Why: the onboarding survey (`docs/context/onboarding-data-contracts.md` §3)
found data-contracts already following this and bridle's docs doing the same.
Whether a checker enforces it is an open question (Q7).

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
