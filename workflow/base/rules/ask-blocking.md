---
id: ask-blocking
severity: should
roles: [orchestrator, project-manager, manager]
---
Ask only questions that block the work, about two a round, each with your
recommendation attached.

- **Blocking means you cannot proceed sensibly without the answer.** Decide the
  rest yourself and say what you assumed.
- **Recommend.** State what you would do and why, so the human can answer
  "yes" rather than design it.

Why: data-contracts' `driver-guide.md` §2. A long list of questions costs the
human more than a wrong small guess costs to fix.

Until `bridle workflow sync` renders rules into agents, the role prompts in the
`bridle` repo's `workflow/base/roles/` carry this.
