---
id: br-6wff
title: "Scratch files in a project's clone are not ignored: .bridle/tmp, reports and review lists show as untracked"
kind: chore
created: 2026-10-22
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-6gdp]
related: [br-6gdp]
---

## The ask

Part 3 of 3 of the orchestrator's finding (br-6gdp). Scratch in the clone is not gitignored, so it shows as untracked and looks like lost work: `.bridle/tmp/` (advisor drafts), `.bridle/overnight-load-report.md` (2026-10-11), `.bridle/review-documents.txt`.

Do: (1) add `.bridle/tmp/` to bridle's own .gitignore and to the .gitignore that project migrations write for every project (a new migration, since projects' files change; existing projects pick it up with `bridle project migrate`); (2) decide for the two stray files: commit them as docs under docs/ if worth keeping (overnight-load-report), else delete; and say in the role texts (advisor, orchestrator) that scratch goes under `.bridle/tmp/`.

Acceptance: `git status` clean of those paths on bridle; a migration test shows `.bridle/tmp/` ignored in a fresh project. `just check`. Model: Haiku or Sonnet. Files: .gitignore, the migrate module (same as the migration-commit sibling ticket: run after it), workflow/base/roles/advisor.md.
