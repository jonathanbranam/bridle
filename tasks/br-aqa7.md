+++
id = "br-aqa7"
title = "Self-upgrade refused good builds 3 times overnight: the new binary's self-check timed out under load"
kind = "incident"
state = "pending"
created_at = "2026-10-05T12:17:39.961Z"
updated_at = "2026-10-05T12:21:10.605287Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Potential incident, filed by the orchestrator at the human's request (2026-10-05 ~8:15 AM ET: 'please do record an incident ... that's a critical kind of thing we want to be sure doesn't happen in the future').

What happened: the daemon's automatic upgrade built green main and then refused to restart into it because the new binary's self-check timed out (60 s), at 03:51Z (0d69d617), 07:19Z (c2ac03d68) and 09:50Z (1f0a5bd43). The self-check passed by hand. Between failures it succeeded once (d1ed064d at 04:48Z) and finally at 12:08Z (1ca6c3c4). br-up82 (retry the self-check on timeout) landed ~06:45Z but couldn't help: the old, running daemon performs the check, so the fix only applies from the first upgrade after it is installed.

Impact: the installed daemon/gateway stayed on d1ed064d for ~7 h while 8 overnight tasks landed; the gateway didn't serve the new Tasks/System/links/send routes, so the bridle-ui pages installed overnight showed no data, and the new web/mobile rule packs weren't active. A human to-do (br-46me) was filed for a manual upgrade.

Cause (to confirm in the postmortem): the self-check's 60 s budget on the Intel Mac under concurrent worker builds/tests (same load family as br-ksz7's cli_e2e timeouts). A fix to the check can't fix the checker that's already running: the bootstrap gap.

Follow-ups: br-up82 (landed), ticket up82, docs/context/incidents.md entry (e0fbe4d0), postmortem ticket (to be filed).

## Thread

### note · external:orchestrator · 2026-10-05T12:21:10.605Z
Postmortem: ticket q7mv (docs/tickets/open/postmortem-the-self-upgrade-refused-three-good-builds-overnight-q7mv.md). Corrections to the body: (1) the gateway (pid 60915, started 2026-10-04 22:05Z) predates br-bek3, so it does NOT follow the upgrade and still serves old code: it needs 'bridle gateway --detach' once (br-46me step 2). (2) br-46me was filed at 03:00Z, before this incident. (3) The 12:08Z success was the old single 60 s check passing on a quiet machine, not up82's retry.
