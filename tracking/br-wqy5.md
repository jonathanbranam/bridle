+++
id = "br-wqy5"
title = "Incidents are only down or stuck systems: a red CI build, a flaky test or a bug is a bug ticket, not an incident"
kind = "chore"
state = "claimed"
created_at = "2026-10-20T23:18:35.450Z"
updated_at = "2026-10-21T01:17:04.550601Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wqy5",
]
ticket = "wqy5"
+++

docs/tickets/open/incidents-are-only-down-or-stuck-systems-a-red-ci-build-a-fl-wqy5.md

## Thread

### note · external:aide · 2026-10-20T23:18:42.701Z
Readied by aide (chore, the human's direct ask; rule ready-without-asking).

### note · external:orchestrator · 2026-10-21T00:55:55.152Z
orchestrator: this ticket's file landed on main in cef10682, whose message wrongly reads 'Revert incident entries 49e3a54b ...' (my git amend slip, 2026-10-20 23:19Z; it was pushed before I corrected it). The content is aide's original ticket commit; the actual revert is 352bb54e. History left as is (pushed).
