+++
id = "br-grdg"
title = "wait-for-wake breaks against daemons older than br-2672: empty message wakes, 403 on meta-notes"
kind = "bug"
state = "planned"
created_at = "2026-10-08T01:02:17.653Z"
updated_at = "2026-10-08T01:02:57.363995Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Found by orchestrator 2026-10-07 ~9:05 PM ET. Since br-2672 (bc5ed0a5), 'bridle orchestrator wait-for-wake' calls GET /v1/wake?principal=external:orchestrator. A daemon built before bc5ed0a5 treats that as a plain principal wake: it answers {reasons:[{reason:"message", messages:[...]}]} with no text/detail and marks the messages read, so the CLI prints 'message: ' and 'null' and the message body is lost (seen on bridle at 5351b6df, bridle-ui, track-web). meta-notes' daemon on the NUC answers 403 'a principal may only wait for its own wake'. The commit says the old route was kept, but the CLI no longer uses it, so every daemon that isn't upgraded yet (other projects, other machines) blinds the orchestrator.
Fix: the CLI falls back to GET /v1/orchestrator/wake when the daemon doesn't serve the new form (e.g. use the old route unless the daemon says it supports the new one, or on 403 or a reason with no text). Simplest acceptable: wait-for-wake keeps calling /v1/orchestrator/wake until every daemon is known to be past bc5ed0a5. Add a test with an old-format reply. Sonnet, small. Workaround meanwhile: the orchestrator calls /v1/orchestrator/wake directly.

## Thread

### note · external:orchestrator · 2026-10-08T01:02:30.999Z
Critical (orchestrator's call): it blinds the orchestrator on every daemon not yet past bc5ed0a5 (bridle-ui, track-web on dalek; meta-notes on the NUC). bridle's own daemon is at bc5ed0a5 now and works. Takes the next free slot, ahead of br-ezpj only if ezpj hasn't started.

### note · external:orchestrator · 2026-10-08T01:02:31.021Z
From orchestrator: CRITICAL, br-grdg is ready. Since br-2672 the CLI's wait-for-wake only calls the new route; older daemons answer empty message wakes (bodies lost, marked read) or 403. Make the CLI fall back to /v1/orchestrator/wake. Next free slot, Sonnet, small. Details on the task.
