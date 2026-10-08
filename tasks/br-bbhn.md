+++
id = "br-bbhn"
title = "Incident: a NUC aide's message to bridle sat undelivered in the outbox: one try timed out while dalek slept, and the outbox never retries (3haz)"
kind = "incident"
state = "dropped"
created_at = "2026-10-08T12:46:23.404Z"
updated_at = "2026-10-08T12:49:16.532999Z"
created_by = "external:aide"
watchers = ["external:aide"]
+++

Filed by the bridle aide at the human's request, 2026-10-08 ~8:20 AM ET. The human: "Did you get a message from the NUC this morning or not? ... The message was sent when the machine was closed. The laptop was sleeping, but it's open now, and that delivery should have occurred. If that message did not get delivered, then file an incident and investigate the cause."

## What happened
The meta-notes aide on the NUC sent a note to `external:aide` on bridle (dalek) at 11:49:50 UTC (7:49 AM ET). It is outbox `o-0032` in the NUC's meta-notes daemon (`/srv/shared/work/meta-notes-work/.bridle/bridle.db`): `state=queued`, `attempts=1`, `last_error="timed out"`, never delivered. dalek's bridle daemon has no message from `external:aide@nuc` today. The body carries the human's request (via the NUC aide) for bridle-ui: view documents on the phone, comment on documents, reply to any task.

## Cause (verified)
dalek (a laptop) was asleep, so the one forward attempt to dalek's daemon timed out. Retrying is not built: principals.md "Mail between daemons (3haz, slice 1)" says "Not built yet: the retry loop and start-up ping (a queued message is retried only when the next message for the same destination is sent)". The NUC's meta-notes daemon has sent nothing to bridle since, so o-0032 sits queued indefinitely and nobody is told. Queued count on that daemon: 1.

## Impact
The human's words from the NUC were silently lost for 30+ min, and indefinitely if no other message goes from that daemon to bridle. The sender thinks it was sent. Any sleeping or restarting receiver (dalek laptop sleep, daemon upgrades) hits this.

## Follow-up
The retry loop (periodic flush with backoff, plus a flush when a destination comes back) from 3haz's remaining slices; also some way to see a stuck outbox (status line, `bridle outbox`). Workaround: the aide on dalek relays the stuck message's content by hand.

## Thread

### note · external:orchestrator · 2026-10-08T12:46:41.758Z
Same root, second case (orchestrator, overnight): from dalek's bridle daemon, 'bridle --project bridle-ui send agent:manager-2 ...' printed 'queued o-0009/10/11'; all three ended state=failed, last_error 'no such recipient: agent:manager-2' (the remote wants 'manager-2'). The sender never hears of the failure, so a work-unblocking nudge was lost for ~7 h. The fix should cover both: a stuck or failed outbox entry must reach its sender (a message back, or the wake), not just a retry. Also worth accepting 'agent:<name>' across daemons as it is locally.

### note · external:orchestrator · 2026-10-08T12:49:16.532Z
dropped: Duplicate of planned work: the fix is br-fvkq (3haz slice 3: outbox retry with backoff, start-up ping), now extended with this incident's requirements: a failed or stuck entry messages its sender, agent:<name> works across daemons, a resume from sleep counts as a start-up, and the send reports the first try's result. Visible outbox state and the human report stay in br-cufw (slice 4). Cause recorded above and on br-fvkq's thread. The stuck NUC message (o-0032) was relayed by hand by aide.
