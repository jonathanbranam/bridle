+++
id = "br-uysm"
title = "Orchestrator relaunch re-runs the pane's previous launcher (respawn-pane with no command), so a moved orchestrator came back as --project bridle"
kind = "bug"
state = "built"
created_at = "2026-10-18T02:10:19.003Z"
updated_at = "2026-10-18T04:12:33.716016Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wuysm",
]
branch = "bridle/wuysm"
commit = "3909360ba44e9a6eea02b6ef3cfd6d01589f20ba"
summary = """Tmux::respawn now takes an explicit command (was Option, None re-ran the pane's previous launcher). First-attempt relaunch respawns PLAIN_SHELL (exec "${SHELL:-sh}") before typing the launcher. Test asserts the respawn command is never tmux's implicit one. CHANGELOG added."""
ticket = "uysm"
+++

docs/tickets/open/orchestrator-relaunch-re-runs-the-pane-s-previous-launcher-r-uysm.md

## Thread

### note · external:aide · 2026-10-18T02:13:52.279Z
From the human, via aide (2026-10-17 ~10:10 PM ET), on leaving the orchestrator running as --project bridle (waiting on dalek-home) until br-uysm lands rather than fixing pane %4 by hand now: "Ageee" (agree).

### note · agent:wuysm · 2026-10-18T04:12:33.716Z
built: 3909360ba44e9a6eea02b6ef3cfd6d01589f20ba (branch bridle/wuysm)
