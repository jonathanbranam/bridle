---
id: br-p3vy
title: Restart prompt reaches the replacement session when the old one hands over the same second
kind: bug
created: 2026-11-01
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-5hca]
---

## The ask

Reported by orchestrator@vesta (m-0562 on dalek-home), from the aide on the notes project (m-0141 on vesta-home). Follow-up to br-5hca (integrated 7085fa99), which marks the handover request read once the handover note lands.

What happened (2026-11-01 ~8:02 AM ET): aide's handover prompt (notes m-0186, "The human is restarting this session") arrived in the same second the old session (pid 1321603) wrote handover h-0031, so the old session never read it. The new session's first waiter (pid 1522933) then got the prompt and was asked to hand over right after starting. Aide ignored it as stale; a less careful session would hand over again for no reason.

Likely cause: br-5hca marks the request read when the note lands, but here the handover was recorded just before (or in the same instant as) the prompt was queued, so the mark ran first and the prompt was delivered afterwards. Order must not matter.

Fix: make the two orderings equivalent. When queueing the restart prompt, skip it (or queue it already read) if that session's handover is already recorded; and when the handover lands, mark any pending prompt read (br-5hca's part). Better still, scope the prompt to the session it was meant for (its session id) so a replacement never receives it, if the existing schema allows it cheaply (read how br-5hca did it first; do not redesign inbox keying).

Acceptance: a test that records the handover first, then queues the prompt (and the reverse order, which br-5hca covers): in both, the replacement session's first wake does not include the prompt; `just check`. Model: Sonnet. Files: the session restart / handover code br-5hca changed, and its tests. Out of scope: how inboxes are keyed in general.
