+++
id = "br-ce1a"
title = "Dropping a claimed task leaves its claim behind (b5br)"
kind = "bug"
state = "open"
created_at = "2026-09-29T02:24:40.607Z"
updated_at = "2026-09-29T02:24:40.607Z"
+++

ticket: docs/questions/open/dropping-a-claimed-task-leaves-its-claim-b5br.md
original id: b5br

The human wants this fixed, and br-29f9's stale claim cleared from the queue.

Bug: `bridle task drop` on a claimed task leaves its `claims` row, so `bridle queue` lists a dropped task under "Claimed:" (br-29f9, agent:branch-rules). `done_task` releases the claim; `drop_task` (crates/bridle-daemon/src/tasks.rs) doesn't. `tick_claim_lease_check` never clears it once the agent is removed, and if the agent still existed, its `release_claim` would set a dropped task back to `planned`. Nobody but the claimant can `bridle release`.

Scope: drop_task releases any claim (keeping the task dropped); the lease check never moves a task that isn't `claimed`; on daemon load, discard a claims row whose task isn't `claimed` (clears br-29f9 on the next restart). Tests for each. Update docs/design/storage.md (claims) if behaviour described there changes.

After it lands: `bridle task reopen br-29f9` then `bridle task done br-29f9 --commit c533cb0` (its work landed; it should read integrated).

Acceptance: just check passes; br-29f9 no longer shows under Claimed in `bridle queue` after a daemon restart on the fixed build.
Model: Sonnet. Size: S.
