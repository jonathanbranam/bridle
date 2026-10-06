+++
id = "br-2y3m"
title = "git push to origin rejected: local main 23 ahead and diverged from origin/main (non-fast-forward)"
kind = "incident"
state = "pending"
created_at = "2026-10-06T02:46:34.626Z"
updated_at = "2026-10-06T02:46:40.287652Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Found 2026-10-05 ~10:50 PM ET by the human ('Is git push failing? ... I saw this several times in other messages'). Local main in /Volumes/Data/work/bridle/bridle is 23 commits ahead and diverged from origin/main; 'git push' is rejected as non-fast-forward. The human (verbatim): 'while I'm sleeping, write up an incident report, investigate this, explain what happened, and make a postmortem. Somebody did something wrong here.' Wanted by 6:00 AM ET. Investigate without changing main or origin (no push, no rebase, no reset): which commits are on origin but not local, who or what pushed them (CI? another machine? a worker?), when it diverged, which agents hit the rejection and what they did, why nobody raised it. Write the postmortem as a ticket (like q7mv), with a timeline, root cause, why it went unnoticed, recommendations and the exact steps to reconcile, for the human to approve in the morning.

## Thread

### note · agent:pm-1 · 2026-10-06T02:46:40.287Z
pm-1: this is pending, so I can't plan it. Orchestrator: run 'bridle task ready br-2y3m' (it is urgent, wanted by 6:00 AM ET; skip the settling wait if you can). Then I will plan it: Sonnet, read-only (no push/rebase/reset of main or origin), deliverable a postmortem ticket via 'bridle ticket new', and put it in tier 1 ahead of br-stx8. Reply with the ticket ID to the orchestrator when the manager reports done.
