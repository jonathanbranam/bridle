+++
id = "br-ec04"
title = "Gateway 3/10: list the human's to-dos and task questions across projects"
kind = "feature"
state = "integrated"
created_at = "2026-10-02T23:35:47.667Z"
updated_at = "2026-10-03T00:37:45.132062Z"
size = "M"
branch = "bridle/gateway-list"
commit = "2f425545d21d73a286ae75a0ae7bef442c86e606"
summary = "GET /api/v1/items (crates/bridle-gateway/src/items.rs): per reachable project, the human's to-dos (claimed_by=human, state claimed) and decisions (every open task question; the daemon has no per-question recipient, so all are the human's to answer), each high priority first then oldest; projects with decisions sort first; unreachable projects (and ones whose reads fail) listed under unreachable. Gateway-owned types with ts-rs, bindings regenerated. Tokens resolve via bridle_api resolve_token (local workspace human token); remote machines need BRIDLE_TOKEN until per-machine tokens land. discovery.rs now exposes its targets (with workspace) to share."
+++

Read docs/design/human-web-ui.md sections 1, 2 and 5 first. Goal: GET /api/v1/items (name as fits): the human's to-dos (daemon GET /v1/tasks?claimed_by=human) and open task questions to the human, from every reachable project (use the discovery from gateway 2), grouped by project, decisions first, each list high priority then oldest; unreachable projects reported alongside. Uses bridle-api; no new daemon endpoints. Gateway-owned response types (they get ts-rs in task 7, so keep them plain serde structs, no daemon types leaking). Tests: ordering, grouping, two projects merged, an unreachable one reported. Acceptance: just check passes. Model: Sonnet. Migration: none. Out of scope: actions, retract hiding (task 5), login. Depends on gateway 2.

## Thread

### note · agent:gateway-list · 2026-10-03T00:34:26.229Z
done: GET /api/v1/items, to-dos and decisions across projects with unreachable reported; d9ded221272e097a003786f32e0ac686d44a6284

### note · agent:manager-2 · 2026-10-03T00:34:30.740Z
Your done report didn't say just check passed. Merge main (gateway-ui-serve, br-fa63, lands first and touches lib.rs/types.rs; I'll tell you when; for now merge current main), run just check, and message me the sha plus the check result.

### note · agent:manager-2 · 2026-10-03T00:35:44.670Z
br-fa63 landed as f360d56 (touches lib.rs, types.rs, config.rs, bindings). Merge main again, regenerate types if needed, run just check, message me the sha and result.

### note · agent:gateway-list · 2026-10-03T00:35:46.756Z
done: just check PASSED (exit 0, 1034 tests, 0 failed) on current main merged in; checked commit 6ce17a435a5713004b92e611138d3e2885fbf934. Will re-merge when you say br-fa63 landed.

### note · agent:gateway-list · 2026-10-03T00:37:30.076Z
done: merged main (br-fa63 f360d56; one conflict in types.rs, both imports kept, bindings current). just check PASSED (exit 0, 1039 tests, 0 failed); checked commit 474207667ad3ee5c3ab3eefdf66aba4cab8e58af

### note · agent:manager-2 · 2026-10-03T00:37:32.445Z
integrated: 2f425545d21d73a286ae75a0ae7bef442c86e606 (branch bridle/gateway-list)

### note · agent:manager-2 · 2026-10-03T00:37:45.132Z
cleanup: removed agent gateway-list, branch bridle/gateway-list
