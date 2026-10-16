+++
id = "br-8rdn"
title = "7r2h gaps: token pair on a machine with no home keeps direct tokens; make re-pair a prompt step; startup check names projects that do not forward"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T10:03:32.380Z"
updated_at = "2026-10-16T22:30:08.282703Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:8rdn",
    "agent:8rdnb",
]
branch = "bridle/8rdnb"
commit = "60c9f9fa23966e2ffeaa9d59a06c5b743e3e3148"
summary = "Item 3 of br-8rdn: `bridle token check` (text mode) now also prints a note naming, on a machine with a <machine>-home, the projects whose orchestrator visitor token is missing or refused (they do not forward to the home). Reuses check results, so no new daemon code or status field. The orchestrator role runs `bridle token check` at every start. Test, cli.md and CHANGELOG updated. JSON output unchanged (array shape kept)."
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

### note · agent:8rdnb · 2026-10-16T21:56:41.460Z
built: 7833e81c (branch bridle/8rdnb)

### note · agent:8rdnb · 2026-10-16T21:56:41.520Z
done: token check names non-forwarding projects incl. missing visitor token with a home; orchestrator runs it at start; check exit 0, 1771 tests; 7833e81c

### note · agent:manager-2 · 2026-10-16T22:29:49.989Z
integrated: 60c9f9fa23966e2ffeaa9d59a06c5b743e3e3148 (branch bridle/8rdnb)

### note · agent:manager-2 · 2026-10-16T22:30:08.282Z
cleanup: removed agent 8rdnb, branch bridle/8rdnb
