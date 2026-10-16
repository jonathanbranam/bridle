+++
id = "br-8rdn"
title = "7r2h gaps: token pair on a machine with no home keeps direct tokens; make re-pair a prompt step; startup check names projects that do not forward"
kind = "fix"
state = "claimed"
created_at = "2026-10-16T10:03:32.380Z"
updated_at = "2026-10-16T21:17:48.827511Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:8rdn",
    "agent:8rdnb",
]
branch = "bridle/8rdn"
commit = "b80a625795d0bb7ac69ab2aad0d9fdacf141fc26"
summary = "On a machine with no <machine>-home, `bridle token pair` and `bridle token check` now end with a note naming the projects whose daemons do not forward wakes/mail to the orchestrator (all but the one it waits on) and saying to create the home and re-run `token pair` (token_pair.rs: non_forwarding, write_non_forwarding; test added). The orchestrator role prompt says to treat those projects as unwatched and make the re-pair a handover step. Docs: cli.md, CHANGELOG. Caveat: item 3 is done as this note, not a `bridle status` field (that would change the wire type and needs the daemon to know the fleet); say if you want the field."
ticket = "8rdn"
+++

docs/tickets/open/7r2h-gaps-token-pair-on-a-machine-with-no-home-keeps-direct-8rdn.md

## Thread

### note · agent:8rdn · 2026-10-16T10:53:12.110Z
built: 4704388c (branch bridle/8rdn)

### note · agent:8rdn · 2026-10-16T10:53:12.171Z
done: token pair/check note naming non-forwarding projects, orchestrator prompt re-pair step, test, docs; check exit 0, 1756 tests; 4704388c

### note · agent:manager-2 · 2026-10-16T11:22:32.769Z
integrated: b80a625795d0bb7ac69ab2aad0d9fdacf141fc26 (branch bridle/8rdn)

### note · agent:manager-2 · 2026-10-16T11:22:53.796Z
cleanup: removed agent 8rdn, branch bridle/8rdn

### note · external:advisor/product-manager · 2026-10-16T21:08:26.536Z
advisor (product-manager), verifying b80a6257: item 2 met. Gap in item 3: no startup check or status field. The only change is a note: from token pair/check, printed when the machine has no home; the orchestrator's startup never runs it. A project missing its visitor token on a machine WITH a home is never named. Reopened for item 3 (this is also br-7r2h's last open item).

### note · agent:pm-1 · 2026-10-16T21:08:36.952Z
Brief for the remainder (pm-1): item 3 of the ask (also br-7r2h's last item). Today only a note is printed by token pair/check, and only when the machine has no home. Build a check run at the orchestrator's startup (or a field in `bridle status --json`; pick whichever reuses existing code with least change, say which) that names every project that does not forward to the home daemon, including a project missing its visitor token on a machine WITH a home. Tests for both cases. Docs: cli.md. Acceptance: just check. Model: Sonnet. Out of scope: new forwarding mechanisms.
