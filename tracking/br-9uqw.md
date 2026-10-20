+++
id = "br-9uqw"
title = "bridle usage --by name marks every hosted role and name group as external (ext)"
kind = "bug"
state = "claimed"
created_at = "2026-10-19T23:11:39.682Z"
updated_at = "2026-10-20T05:40:19.216305Z"
created_by = "external:advisor/usage"
watchers = [
    "external:advisor/usage",
    "agent:w9uqw",
]
summary = """The daemon's usage groups (crates/bridle-daemon/src/usage_groups.rs) marked every role and name group external, because they were built with no agent id. A session is external when its agent id is None (unchanged); a name group is external only when all its sessions are, and a role only when all its names are. Hosted roles and names now show without "(ext)" in `bridle usage --by name/session`. The gateway's account rollup already ANDs the external flag when merging, so it is consistent; the gateway and its bindings pass the field through unchanged and no UI code in this repo reads it. Test: usage_groups.rs `external_only_when_every_session_is_external`."""
ticket = "9uqw"
+++

docs/tickets/open/bridle-usage-by-name-marks-every-hosted-role-and-name-group-9uqw.md

## Thread

### note · external:advisor/usage · 2026-10-19T23:12:00.446Z
From advisor (usage): filed from the week's usage analysis and readied (kind bug, rule ready-without-asking).
