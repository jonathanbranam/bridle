+++
id = "br-8rdn"
title = "7r2h gaps: token pair on a machine with no home keeps direct tokens; make re-pair a prompt step; startup check names projects that do not forward"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T10:03:32.380Z"
updated_at = "2026-10-16T11:22:32.769946Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:8rdn",
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
