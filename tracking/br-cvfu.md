+++
id = "br-cvfu"
title = "bridle daemon restart --cancel / --postpone: hold a pending drain while a land runs (pulled forward from 8swr)"
kind = "feature"
state = "integrated"
created_at = "2026-10-14T22:15:24.125Z"
updated_at = "2026-10-15T00:08:46.115716Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:cvfu",
]
branch = "bridle/cvfu"
commit = "1d74af107213c05d1154f819fd65e167bb9180f8"
summary = "Added POST /v1/restart/hold (RestartHoldRequest {cancel, postpone_secs}) and `bridle daemon restart --cancel | --postpone <dur>`. Allowed: human, external:orchestrator, manager-role agents. Cancel flags the pending drain loop, which lifts the drain (the waiting restart request gets a 'cancelled' 409); an upgrade drain ends quietly with an upgrade.cancelled event, no failure wake, and the commit is not recorded built. Postpone is an in-memory expiry (cap 7d) on AgentManager: the self-upgrade tick skips, new restart/--upgrade requests get 409, and a pending drain won't take its quiet point until expiry. Status gains postponed_until and prints 'upgrade    postponed until HH:MM'. Caveat: postpone does not lift a drain (claims/spawns stay refused), so postpone before the drain. Docs: daemon.md, cli.md, CHANGELOG, manager role landing step. Tests: tests/restart_hold_test.rs (5)."
ticket = "cvfu"
+++

docs/tickets/open/bridle-daemon-restart-cancel-postpone-hold-a-pending-drain-w-cvfu.md

## Thread

### note · agent:cvfu · 2026-10-14T23:56:36.712Z
done: bridle daemon restart --cancel/--postpone (POST /v1/restart/hold, status postponed_until, docs, CHANGELOG, manager role step, 5 new tests); just check exit 0, 1691 tests passed; 393319d6 (main already merged, up to date)

### note · agent:manager-2 · 2026-10-14T23:57:31.371Z
built: 393319d6 (branch bridle/cvfu)

### note · agent:manager-2 · 2026-10-15T00:08:46.115Z
integrated: 1d74af107213c05d1154f819fd65e167bb9180f8 (branch bridle/cvfu)
