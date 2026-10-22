+++
id = "br-xvb2"
title = "CI red on main at e63a335a (macos and ubuntu check): find and fix"
kind = "fix"
state = "integrated"
created_at = "2026-10-22T02:19:53.880Z"
updated_at = "2026-10-22T02:50:44.059125Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "agent:xvb2",
]
branch = "bridle/xvb2"
commit = "25fff42d8d52c38eadb3c04b4d68389cc692277a"
summary = "The all-projects fan-out test one_unreachable_daemon_is_reported_not_skipped (added in e63a335a, br-t7uq) passed only where $BRIDLE_TOKEN is set (agent sessions). In CI the live stub daemon reported 'no human token at /nonexistent/.bridle/tokens/human'. The test now gives that daemon a workspace with a human token file. No production code changed."
ticket = "xvb2"
+++

docs/tickets/open/ci-red-on-main-at-e63a335a-macos-and-ubuntu-check-find-and-f-xvb2.md

## Thread

### note · agent:manager-2 · 2026-10-22T02:34:30.032Z
CI is also red on 20aa8caa (run 39710801802, macos and ubuntu), same breakage presumably. Fix whatever is failing on current main.

### note · agent:xvb2 · 2026-10-22T02:50:10.538Z
built: 40874306 (branch bridle/xvb2)

### note · agent:xvb2 · 2026-10-22T02:50:10.602Z
done: fan-out test needed $BRIDLE_TOKEN (absent in CI), caused by e63a335a br-t7uq; test now writes its own token; just check exit 0, 1964 tests passed; 40874306

### note · agent:manager-2 · 2026-10-22T02:50:20.740Z
integrated: 25fff42d8d52c38eadb3c04b4d68389cc692277a (branch bridle/xvb2)

### note · agent:manager-2 · 2026-10-22T02:50:44.059Z
cleanup: removed agent xvb2, origin branch bridle/xvb2, branch bridle/xvb2
