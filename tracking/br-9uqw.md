+++
id = "br-9uqw"
title = "bridle usage --by name marks every hosted role and name group as external (ext)"
kind = "bug"
state = "integrated"
created_at = "2026-10-19T23:11:39.682Z"
updated_at = "2026-10-20T05:58:22.532644Z"
created_by = "external:advisor/usage"
watchers = [
    "external:advisor/usage",
    "agent:w9uqw",
]
branch = "bridle/w9uqw"
commit = "7bf8cf859a94a105501d8cdcab4135f1a6808140"
summary = """The daemon's usage groups (crates/bridle-daemon/src/usage_groups.rs) marked every role and name group external, because they were built with no agent id. A session is external when its agent id is None (unchanged); a name group is external only when all its sessions are, and a role only when all its names are. Hosted roles and names now show without "(ext)" in `bridle usage --by name/session`. The gateway's account rollup already ANDs the external flag when merging, so it is consistent; the gateway and its bindings pass the field through unchanged and no UI code in this repo reads it. Test: usage_groups.rs `external_only_when_every_session_is_external`."""
ticket = "9uqw"
+++

docs/tickets/open/bridle-usage-by-name-marks-every-hosted-role-and-name-group-9uqw.md

## Thread

### note · external:advisor/usage · 2026-10-19T23:12:00.446Z
From advisor (usage): filed from the week's usage analysis and readied (kind bug, rule ready-without-asking).

### note · agent:w9uqw · 2026-10-20T05:57:52.370Z
built: d0b52b39 (branch bridle/w9uqw)

### note · agent:w9uqw · 2026-10-20T05:58:09.581Z
done: role and name groups are (ext) only when all their sessions are external; hosted roles/names show plain; test external_only_when_every_session_is_external; CHANGELOG Fixed line; summary set; main already merged; just check exit 0, 1939 passed, 6 skipped; tip d0b52b39 (branch bridle/w9uqw, pushed)

### note · agent:manager-2 · 2026-10-20T05:58:18.330Z
integrated: 7bf8cf859a94a105501d8cdcab4135f1a6808140 (branch bridle/w9uqw)

### note · agent:manager-2 · 2026-10-20T05:58:22.532Z
cleanup: removed nothing; kept agent w9uqw (background job pid 33548)
