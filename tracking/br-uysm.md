+++
id = "br-uysm"
title = "Orchestrator relaunch re-runs the pane's previous launcher (respawn-pane with no command), so a moved orchestrator came back as --project bridle"
kind = "bug"
state = "integrated"
created_at = "2026-10-18T02:10:19.003Z"
updated_at = "2026-10-18T05:08:57.245927Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wuysm",
]
branch = "bridle/wuysm"
commit = "3b7bb078b2b8effd4aff17f3ceb30f698b37f8bd"
summary = """Tmux::respawn now takes an explicit command (was Option, None re-ran the pane's previous launcher). First-attempt relaunch respawns PLAIN_SHELL (exec "${SHELL:-sh}") before typing the launcher. Test asserts the respawn command is never tmux's implicit one. CHANGELOG added."""
ticket = "uysm"
+++

docs/tickets/open/orchestrator-relaunch-re-runs-the-pane-s-previous-launcher-r-uysm.md

## Thread

### note · external:aide · 2026-10-18T02:13:52.279Z
From the human, via aide (2026-10-17 ~10:10 PM ET), on leaving the orchestrator running as --project bridle (waiting on dalek-home) until br-uysm lands rather than fixing pane %4 by hand now: "Ageee" (agree).

### note · agent:wuysm · 2026-10-18T04:12:33.716Z
built: 3909360ba44e9a6eea02b6ef3cfd6d01589f20ba (branch bridle/wuysm)

### note · agent:wuysm · 2026-10-18T04:27:03.190Z
done: relaunch respawns an explicit shell (PLAIN_SHELL), Tmux::respawn takes a required command; just check exit 0, 1851 tests passed, includes main merge; 5a76b7b2ab3a1e94629ddfb67d065bfcca8ce009

### note · agent:manager-2 · 2026-10-18T05:08:42.820Z
integrated: 3b7bb078b2b8effd4aff17f3ceb30f698b37f8bd (branch bridle/wuysm)

### note · agent:manager-2 · 2026-10-18T05:08:57.245Z
cleanup: removed agent wuysm, branch bridle/wuysm
