+++
id = "br-7zd4"
title = "Remove the stale 'Until bridle workflow sync renders rules' lines from workflow/base/rules (98xt)"
kind = "chore"
state = "planned"
created_at = "2026-10-09T23:27:48.286Z"
updated_at = "2026-10-11T03:43:46.927339Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "external:advisor/workflow",
]
priority = "low"
priority_at = "2026-10-09T23:27:48.287084Z"
summary = "Removed the stale 'Until bridle workflow sync renders rules...' paragraph and the 'Agents read the rule files themselves...' paragraph from 14 workflow/base/rules files (the brief said about 13); the rest of each rule is unchanged. Worker role intro drops the 'see CLAUDE.md bridle block' sentence (no such block). The no-kill-by-name wake heading no longer says 'not built yet': supersede (exit 5) and 'wake --stop' are in the code. Vendored copies: .bridle/roles/advisor.md and .bridle/roles/orchestrator.md are tracked in this repo; left unedited per the brief (base is the source). Prime recommendations 2 and 3 untouched."
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

### note · agent:w7zd4 · 2026-10-11T02:58:16.841Z
Vendored copies: .bridle/roles/advisor.md and .bridle/roles/orchestrator.md are tracked in git in this repo; not edited (workflow/base is the source). just check exit 0, 1484 run / 1484 passed / 5 skipped, on 61f2dd18.

### note · agent:w7zd4 · 2026-10-11T02:58:18.604Z
done: stale rule lines removed from 14 base rules, worker intro and 75h2 heading fixed; just check exit 0, 1484 tests; 61f2dd18

### note · agent:manager-2 · 2026-10-11T03:28:37.740Z
land failed: main moved. Merge main into your branch, re-run just check once, message me the new sha.

### note · agent:w7zd4 · 2026-10-11T03:43:46.927Z
done: main merged again (7fb9df31 into branch); just check exit 0, 1487 tests run, 1487 passed, 5 skipped; b0891647
