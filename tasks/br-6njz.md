+++
id = "br-6njz"
title = "Investigate incident br-2y3m (diverged main, push rejected) and write the postmortem ticket"
kind = "research"
state = "pending"
created_at = "2026-10-06T02:48:34.028Z"
updated_at = "2026-10-06T02:48:35.881464Z"
created_by = "agent:pm-1"
watchers = ["agent:pm-1"]
+++

Linked to incident br-2y3m (read its thread: the human's words and the added scope). Overnight, wanted by 6:00 AM ET 2026-10-06 for the human to approve in the morning. The incident is the record; this task is the work.

HARD RULES: read-only on git. NO push, rebase, reset, merge, checkout of main, fetch --prune or branch deletion, in any clone or worktree. `git fetch origin` is allowed. Edit nothing except the new ticket file. Use `git log`, `git diff`, `git show`, `git merge-base`, `git reflog`, and read-only `gh` calls (`gh run list`, `gh api` GETs only).

INVESTIGATE:
- Commits on origin/main but not local main, and the reverse (`git log --left-right --cherry-pick main...origin/main`); merge base and date; whether the 23 local commits are patch-equivalent to origin ones or genuinely different.
- Who or what made the origin-only commits (author, committer, message, time): CI, another machine (dalek, NUC), a worker, the human, a GitHub UI action. Check .github workflows for anything that pushes to main.
- When it diverged (first commit on each side after the merge base).
- Which agents hit the rejection: search task threads, agent logs and events (`bridle task show`, `bridle events`, `bridle logs` as available) for 'rejected', 'non-fast-forward', 'failed to push', 'ahead of', 'behind', 'push'; what they did (retry, force, ignore, work around).
- WHY WASN'T IT REPORTED TO THE HUMAN (added by the human): several agents mentioned in passing that main was N commits ahead or behind origin and nobody raised an incident. Human: 'every agent should be going, Holy shit, there's an incident. We can't push. If this laptop's hard drive dies, all the work is gone.' Cover: who saw it and when, why nobody filed or raised an incident, why the orchestrator didn't promote one, and how an unpushable main must reach the human and every working agent (recommend a rule and/or a daemon check).
- Who is supposed to push main (docs/design/agent-host/operating-model.md), whether the landing flow or any check verifies the push, and whether failures are silent.

DELIVERABLE: `bridle ticket new "<title>" --kind incident --body-file <file>`, modelled on the postmortem ticket q7mv (find it under docs/tickets/). Sections: Summary, Timeline (UTC; US Eastern in parentheses only for times the human quoted), Root cause, Why it went unnoticed and unreported, What each agent did, Recommendations (cheap fixes first, with a rule or check where fitting), RECONCILE STEPS (exact commands in order to bring local main and origin/main together without losing the 23 local commits or the origin-only ones, with a check after each step and a rollback; do NOT run them). ASCII only. Cite facts by commit sha. Run `bridle ticket check`. Put the new ticket's ID in your done note.

Acceptance: the ticket exists with all sections; no branch or remote changed. `just check` not needed (no code).
Model: Sonnet. Out of scope: reconciling, fixing the cause in code, any push.

## Thread

### note · agent:pm-1 · 2026-10-06T02:48:35.881Z
pm-1: research task for incident br-2y3m, brief includes the human's added scope (why it wasn't reported). It is pending: orchestrator, please run 'bridle task ready br-6njz'; I then plan it and put it in tier 1.
