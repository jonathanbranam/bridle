+++
id = "br-wqy5"
title = "Incidents are only down or stuck systems: a red CI build, a flaky test or a bug is a bug ticket, not an incident"
kind = "chore"
state = "integrated"
created_at = "2026-10-20T23:18:35.450Z"
updated_at = "2026-10-21T02:02:05.768002Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "agent:wqy5",
]
branch = "bridle/wqy5"
commit = "5c19150a8c058d214faf3d39014984cdfa629ec0"
summary = "Narrowed what counts as an incident (the human, 2026-10-20: a red CI build, a flaky test or a bug is not an incident; an incident is a system down or blocked and not moving). Changed: workflow/base/roles/orchestrator.md 'Keep the incident log' (definition, bug/fix ticket for red CI, flakes and bugs); docs/context/incidents.md header (same definition; existing entries untouched); docs/design/agent-host/incidents.md (a 'What counts' paragraph, and the daemon's candidate list now says a main stalled so nothing lands, not a red main). Manager and project-manager roles and rules had no CI-as-incident wording, so they are unchanged. CHANGELOG Changed line added. Caveat: bridle ticket check reports 3 pre-existing problems on other tickets (related ids br-ck8a, br-n7cg, br-a885 not tickets); none on br-wqy5."
ticket = "wqy5"
+++

docs/tickets/open/incidents-are-only-down-or-stuck-systems-a-red-ci-build-a-fl-wqy5.md

## Thread

### note · external:aide · 2026-10-20T23:18:42.701Z
Readied by aide (chore, the human's direct ask; rule ready-without-asking).

### note · external:orchestrator · 2026-10-21T00:55:55.152Z
orchestrator: this ticket's file landed on main in cef10682, whose message wrongly reads 'Revert incident entries 49e3a54b ...' (my git amend slip, 2026-10-20 23:19Z; it was pushed before I corrected it). The content is aide's original ticket commit; the actual revert is 352bb54e. History left as is (pushed).

### note · agent:wqy5 · 2026-10-21T01:38:43.874Z
built: c92577e7 (branch bridle/wqy5)

### note · agent:wqy5 · 2026-10-21T01:38:44.329Z
done: incidents narrowed to down or blocked systems in orchestrator role, incidents log header and incidents design doc; manager and project-manager had no CI-as-incident wording, so unchanged; CHANGELOG Changed line added; just check exit 0 (1942 run, 1942 passed, 7 skipped); bridle ticket check shows 3 pre-existing problems on other tickets, none on br-wqy5; commit c92577e7 (main already merged, no-op)

### note · agent:manager-2 · 2026-10-21T02:02:05.768Z
integrated: 5c19150a8c058d214faf3d39014984cdfa629ec0 (branch bridle/wqy5)
