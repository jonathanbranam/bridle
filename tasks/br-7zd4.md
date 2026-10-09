+++
id = "br-7zd4"
title = "Remove the stale 'Until bridle workflow sync renders rules' lines from workflow/base/rules (98xt)"
kind = "chore"
state = "planned"
created_at = "2026-10-09T23:27:48.286Z"
updated_at = "2026-10-09T23:29:07.800377Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:advisor/workflow",
]
priority = "low"
priority_at = "2026-10-09T23:27:48.287084Z"
parent = "br-0473"
+++

Ticket: docs/tickets/open/role-and-rule-files-misstate-how-rules-reach-agents-orchestr-98xt.md (read "Wrong claims in workflow/" and "Next steps"; only the plain fixes, not the question about prime). Model: Haiku. Docs only.

Do:
1. In workflow/base/rules/*.md remove the stale lines: "Until `bridle workflow sync` renders rules into agents, the role prompts in the `bridle` repo's `workflow/base/roles/` carry this." and "Agents read the rule files themselves (`bridle sync` writes the CLAUDE.md pointer to them; rules are not rendered into the prompt), and the role prompts ... carry this too." Remove the whole paragraph or sentence, keep the rest of each rule. Use `grep -n "Until .bridle workflow sync"` and `grep -n "Agents read the rule files"` across workflow/ to find every one (about 13 files; rules are rendered into the spawn prompt, so the rule body is already in the agent's prompt).
2. workflow/base/roles/worker.md lines ~4-5: drop the second sentence ("The workflow rules named below are files: see CLAUDE.md's bridle block ...") since the rules are in the worker's system prompt and bridle's CLAUDE.md has no such block.
3. The advisor prime text (workflow/base/roles/advisor.md or the prime output source: grep "not built yet" near wait-stopping / 75h2): supersede (exit 5) and `bridle agent wake --stop` work; fix that line to say so.
4. Do NOT cite-by-id rewrites, sync.rs, or prime changes (the ticket's recommendations 2 and 3 are a separate decision). Are these rule files vendored copies in other places (e.g. .bridle/workflow/base/rules/ in this repo)? If bridle's own repo has a vendored copy checked in, say so in the task comment and edit only workflow/base, which is the source.
Acceptance: just check passes (if rule/role text is checked by tests, fix them). Migration: projects get the text via bridle workflow sync. Out of scope: everything above not listed.

## Thread

### note · external:advisor/product-manager · 2026-10-09T23:27:48.287Z
priority: normal -> low
