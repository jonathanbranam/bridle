+++
id = "br-ec5m"
title = "Move output polish: take-over prints only diffs, start drops the contradictory mail hint, runbook names a real mail status command (from jqf9)"
kind = "fix"
state = "integrated"
created_at = "2026-10-18T23:43:07.807Z"
updated_at = "2026-10-20T02:45:41.890535Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:wec5m",
]
branch = "bridle/wec5m"
commit = "843f9a5806ace1deac1ca5429683d3bf2a6e12da"
summary = "take-over now uses project_sync::sync_diffs_only (no table, no 'same' lines); mail_install::install_unit(load_hint=false) used by project start omits the 'nothing was started' text; move-a-project.md step 5.5 names launchctl/systemctl. Tests added for both. The '[mail] allow' column run-together goes away with the table; the diff lines were not changed."
ticket = "ec5m"
+++

docs/tickets/open/move-output-polish-take-over-prints-only-diffs-start-drops-t-ec5m.md

## Thread

### note · agent:wec5m · 2026-10-20T02:45:21.962Z
built: ce561fb8 (branch bridle/wec5m)

### note · agent:wec5m · 2026-10-20T02:45:22.353Z
done: take-over diffs only, start drops mail hint, runbook fixed; check exit 0, 1915 tests; ce561fb8

### note · agent:manager-2 · 2026-10-20T02:45:29.551Z
integrated: 843f9a5806ace1deac1ca5429683d3bf2a6e12da (branch bridle/wec5m)

### note · agent:manager-2 · 2026-10-20T02:45:41.890Z
cleanup: removed agent wec5m, branch bridle/wec5m
