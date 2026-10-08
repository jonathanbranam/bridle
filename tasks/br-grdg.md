+++
id = "br-grdg"
title = "wait-for-wake breaks against daemons older than br-2672: empty message wakes, 403 on meta-notes"
kind = "bug"
state = "integrated"
created_at = "2026-10-08T01:02:17.653Z"
updated_at = "2026-10-08T01:46:15.931251Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
branch = "bridle/waitfallback"
commit = "d4e1ff6f2bb93ab3e098d1286e75f01ba6748390"
summary = "wait-for-wake now calls GET /v1/orchestrator/wake (served by every daemon, same wakes incl. daemon_stopping) instead of GET /v1/wake, which pre-br-2672 daemons answered with text-less message wakes (body lost, marked read) or 403. New unit test in commands/orchestrator.rs against a fake old-daemon server (404 on any other path). CHANGELOG line. Caveat: upgrade_test::a_drain_holds_new_turns... is timing-flaky under heavy load (load 137 failed it twice; passes alone and with NEXTEST_TEST_THREADS=4)."
+++

Found by orchestrator 2026-10-07 ~9:05 PM ET. Since br-2672 (bc5ed0a5), 'bridle orchestrator wait-for-wake' calls GET /v1/wake?principal=external:orchestrator. A daemon built before bc5ed0a5 treats that as a plain principal wake: it answers {reasons:[{reason:"message", messages:[...]}]} with no text/detail and marks the messages read, so the CLI prints 'message: ' and 'null' and the message body is lost (seen on bridle at 5351b6df, bridle-ui, track-web). meta-notes' daemon on the NUC answers 403 'a principal may only wait for its own wake'. The commit says the old route was kept, but the CLI no longer uses it, so every daemon that isn't upgraded yet (other projects, other machines) blinds the orchestrator.
Fix: the CLI falls back to GET /v1/orchestrator/wake when the daemon doesn't serve the new form (e.g. use the old route unless the daemon says it supports the new one, or on 403 or a reason with no text). Simplest acceptable: wait-for-wake keeps calling /v1/orchestrator/wake until every daemon is known to be past bc5ed0a5. Add a test with an old-format reply. Sonnet, small. Workaround meanwhile: the orchestrator calls /v1/orchestrator/wake directly.

## Thread

### note · external:orchestrator · 2026-10-08T01:02:30.999Z
Critical (orchestrator's call): it blinds the orchestrator on every daemon not yet past bc5ed0a5 (bridle-ui, track-web on dalek; meta-notes on the NUC). bridle's own daemon is at bc5ed0a5 now and works. Takes the next free slot, ahead of br-ezpj only if ezpj hasn't started.

### note · external:orchestrator · 2026-10-08T01:02:31.021Z
From orchestrator: CRITICAL, br-grdg is ready. Since br-2672 the CLI's wait-for-wake only calls the new route; older daemons answer empty message wakes (bodies lost, marked read) or 403. Make the CLI fall back to /v1/orchestrator/wake. Next free slot, Sonnet, small. Details on the task.

### note · external:aide · 2026-10-08T01:03:55.556Z
From the human, via aide (~10:05 PM ET): "approve: br-grdg".

### note · agent:manager-2 · 2026-10-08T01:05:41.232Z
br-grdg (critical) can't spawn: load 6.6 per core (threshold 2.5), slot is free. Override with --ignore-budget for this small fix, or wait? Recommend override (it is small and critical; only mailinstall is building). I will start the at2j land (docs-only check) only after the spawn.

### note · agent:waitfallback · 2026-10-08T01:43:50.292Z
done: wait-for-wake uses GET /v1/orchestrator/wake again (works on pre-br-2672 daemons), test + CHANGELOG; cea88d2b; just check exit 0, 1316 tests run, all passed (main merged). Note: upgrade_test a_drain_holds_new_turns... failed twice at machine load 137 (flaky, passes alone / with NEXTEST_TEST_THREADS=4).

### note · agent:manager-2 · 2026-10-08T01:46:07.315Z
integrated: d4e1ff6f2bb93ab3e098d1286e75f01ba6748390 (branch bridle/waitfallback)

### note · agent:manager-2 · 2026-10-08T01:46:15.931Z
cleanup: removed agent waitfallback, branch bridle/waitfallback
