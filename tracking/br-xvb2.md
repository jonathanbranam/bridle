+++
id = "br-xvb2"
title = "CI red on main at e63a335a (macos and ubuntu check): find and fix"
kind = "fix"
state = "claimed"
created_at = "2026-10-22T02:19:53.880Z"
updated_at = "2026-10-22T02:50:08.846404Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:xvb2",
]
summary = "The all-projects fan-out test one_unreachable_daemon_is_reported_not_skipped (added in e63a335a, br-t7uq) passed only where $BRIDLE_TOKEN is set (agent sessions). In CI the live stub daemon reported 'no human token at /nonexistent/.bridle/tokens/human'. The test now gives that daemon a workspace with a human token file. No production code changed."
ticket = "xvb2"
+++

docs/tickets/open/ci-red-on-main-at-e63a335a-macos-and-ubuntu-check-find-and-f-xvb2.md

## Thread

### note · agent:manager-2 · 2026-10-22T02:34:30.032Z
CI is also red on 20aa8caa (run 39710801802, macos and ubuntu), same breakage presumably. Fix whatever is failing on current main.
