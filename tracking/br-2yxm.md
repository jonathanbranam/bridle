+++
id = "br-2yxm"
title = "Self-upgrade starves while the manager lands back to back: each land's --postpone 45m pushes the upgrade out again"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T23:51:08.165Z"
updated_at = "2026-10-20T00:45:12.158051Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w2yxm",
]
branch = "bridle/w2yxm"
commit = "671aad4b4bf5cfa631d6d32dab2b4f6510d3e30c"
summary = "land_task now wraps land_task_inner and calls Manager::end_postponement() when the land ends (any outcome), so a --postpone set before a land does not outlive it and a pending self-upgrade runs between lands. Test: a_land_ends_the_postponement_it_was_made_for (restart_hold_test.rs). Manager role text and daemon.md (Cancel and postpone) updated; CHANGELOG Fixed line."
ticket = "2yxm"
+++

docs/tickets/open/self-upgrade-starves-while-the-manager-lands-back-to-back-ea-2yxm.md

submitted by external:orchestrator

Seen 2026-10-19 ~7-8 PM ET: green-CI upgrade postponed for over an hour (19:08, 19:27, 19:46, 20:06) because manager-2 runs 'bridle daemon restart --postpone 45m' before every land (its handover routine) and lands came every ~20 min. A human was waiting on a landed fix (br-ggbx) going live. Fix idea: postpone only covers the land in progress (ends when the land ends), or a pending upgrade gets a window between lands before the next claim. Workaround used: orchestrator told the manager to pause new work after one land and run the upgrade.

## Thread

### note · external:orchestrator · 2026-10-19T23:51:08.165Z
submitted by external:orchestrator

### note · agent:w2yxm · 2026-10-20T00:36:56.871Z
built: 5ec6417e8956e088b7cfba637a6b703c3724f767 (branch bridle/w2yxm)

### note · agent:w2yxm · 2026-10-20T00:36:57.238Z
done: a postponement ends when the land ends; just check exit 0, 1903 tests; 5ec6417e8956e088b7cfba637a6b703c3724f767

### note · agent:manager-2 · 2026-10-20T00:44:58.205Z
integrated: 671aad4b4bf5cfa631d6d32dab2b4f6510d3e30c (branch bridle/w2yxm)

### note · agent:manager-2 · 2026-10-20T00:45:12.158Z
cleanup: removed agent w2yxm, branch bridle/w2yxm
