+++
id = "br-2y3m"
title = "git push to origin rejected: local main 23 ahead and diverged from origin/main (non-fast-forward)"
kind = "incident"
state = "planned"
created_at = "2026-10-06T02:46:34.626Z"
updated_at = "2026-10-06T02:48:40.665921Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Found 2026-10-05 ~10:50 PM ET by the human ('Is git push failing? ... I saw this several times in other messages'). Local main in /Volumes/Data/work/bridle/bridle is 23 commits ahead and diverged from origin/main; 'git push' is rejected as non-fast-forward. The human (verbatim): 'while I'm sleeping, write up an incident report, investigate this, explain what happened, and make a postmortem. Somebody did something wrong here.'

GOAL: a postmortem ticket, wanted by 6:00 AM ET (2026-10-06), for the human to approve in the morning.

HARD RULES: read-only on git. NO push, rebase, reset, merge, checkout of main, fetch --prune or branch deletion, in any clone or worktree. `git fetch origin` is allowed (it only updates remote-tracking refs). Do not edit anything except the new ticket file. Work from a worktree or the main clone read-only; use `git log`, `git diff`, `git show`, `git merge-base`, `git reflog` and `gh` read-only calls (e.g. `gh run list`, `gh api` GETs only).

INVESTIGATE:
- Commits on origin/main but not local main, and the reverse (`git log --left-right --cherry-pick main...origin/main`); the merge base and its date; whether the 23 local commits are the same changes re-made (patch-equivalent) or genuinely different.
- Who or what created the origin-only commits (author, committer, message, time): CI, another machine (dalek, NUC), a worker, the human, a GitHub UI action. Check .github workflows for anything that pushes to main.
- When it diverged (first commit on each side after the merge base).
- Which agents hit the rejection: search bridle task threads and agent logs/events (`bridle task show`, `bridle events`/`bridle logs` as available) for 'rejected', 'non-fast-forward', 'failed to push', 'push'; what they did about it (retry, force, ignore, work around) and whether anyone told the human or the orchestrator.
- Why nobody raised it: who is supposed to push main (docs/design/agent-host/operating-model.md), whether a check or the landing flow verifies the push, and whether failures are silent.

DELIVERABLE: `bridle ticket new "<title>" --kind incident --body-file <file>` (model it on the postmortem ticket q7mv: find it under docs/tickets/). Body sections: Summary, Timeline (UTC; add US Eastern in parentheses only for times the human quotes), Root cause, Why it went unnoticed, What each agent did, Recommendations (cheap fixes first, with a rule or check where fitting), and RECONCILE STEPS: the exact commands, in order, to bring local main and origin/main together safely without losing the 23 local commits or the origin-only ones, with a check after each step and a rollback; do NOT run them. ASCII only. Run `bridle ticket check`. Then `bridle task done`-style finish as your role requires, and put the new ticket's ID in your done note.

Acceptance: the ticket exists with all sections, facts cited by commit sha, no change to any branch or remote. `just check` is not needed (no code).
Model: Sonnet. Out of scope: reconciling, fixing the cause in code, any push.

## Thread

### note · agent:pm-1 · 2026-10-06T02:46:40.287Z
pm-1: this is pending, so I can't plan it. Orchestrator: run 'bridle task ready br-2y3m' (it is urgent, wanted by 6:00 AM ET; skip the settling wait if you can). Then I will plan it: Sonnet, read-only (no push/rebase/reset of main or origin), deliverable a postmortem ticket via 'bridle ticket new', and put it in tier 1 ahead of br-stx8. Reply with the ticket ID to the orchestrator when the manager reports done.

### note · external:orchestrator · 2026-10-06T02:48:24.225Z
Added by the human (~10:55 PM ET), part of the postmortem: WHY WASN'T THIS REPORTED TO THE HUMAN? Several agents mentioned in passing that main was N commits ahead of or behind origin, and nobody raised an incident. The human: 'every agent should be going, Holy shit, there's an incident. We can't push. If this laptop's hard drive dies, all the work is gone.' Cover: who saw it and when, why nobody filed or raised an incident, why the orchestrator didn't promote one, and how an unpushable main must reach the human and every working agent.

### note · agent:manager-2 · 2026-10-06T02:48:40.665Z
manager-2: cannot spawn the postmortem worker: max_workers 2 reached (dev-site-doc br-dxcw, designer-role br-ukpm). Recommend 'bridle budget max-workers 3' (orchestrator/human call) or wait for a worker to finish; I will spawn as soon as a slot frees. Also for the record: I landed br-xxw9 (65b9c670) at ~01:37 UTC, hit the push rejection, could not fetch (denied), and could not report it: 'bridle send' failed with 'error: unknown:' on every call.
