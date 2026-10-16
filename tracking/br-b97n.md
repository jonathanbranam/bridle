+++
id = "br-b97n"
title = "Setup gap (a): projects add must not drop unregistered [mail] entries when it syncs"
kind = "fix"
state = "integrated"
created_at = "2026-10-16T02:24:03.944Z"
updated_at = "2026-10-16T04:18:10.894452Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:b97n",
]
branch = "bridle/b97n"
commit = "5c8980ccd3b63a81483bf23b574199836beb1d5a"
summary = "projects sync / add now derive [mail] projects as the [projects] names plus every name any machine already lists (kept, and printed as 'left X (not in [projects])'); nothing is removed. Fix in project_sync.rs (mail_projects_keeping), test added, add-a-machine.md sec 6 step 2 and numbering fixed (gap e), CHANGELOG."
ticket = "b97n"
+++

docs/tickets/open/setup-gap-a-projects-add-must-not-drop-unregistered-mail-ent-b97n.md

## Thread

### note · agent:pm-1 · 2026-10-16T02:24:14.333Z
Brief (pm-1). Model: Sonnet. Size: S. Find the sync that rewrites `[mail] projects`: git grep -n "mail" crates/bridle/src/projects*.rs crates/bridle/src/*sync*.rs (br-29se's `projects add` and br-afzu's enrolment). Change it to add enrolments for registered projects and never remove an entry not in `[projects]`; print one line naming entries it left alone. Tests: sync with a [mail] entry for an unregistered project keeps it on every machine; a new project is added; existing behaviour for registered projects unchanged. Docs: docs/context/add-a-machine.md sec. 6 step 2 (projects sync spreads [mail] now) and fix the numbering that skips 6; CHANGELOG. Acceptance: just check. Touches the same files as br-29se/afzu (landed) and possibly project_start.rs (br-btpy): if that is in flight, the manager runs this one first. Out of scope: removing stale mail entries.

### note · external:advisor/product-manager · 2026-10-16T02:24:57.500Z
Readied by advisor product-manager under the human's go on dffz (2026-10-15 ~10:15 PM ET): "Yes dffz. Fix as much as you can tonight".

### note · agent:b97n · 2026-10-16T03:50:30.488Z
built: fd704c42 (branch bridle/b97n)

### note · agent:b97n · 2026-10-16T03:50:30.692Z
done: sync keeps unregistered [mail] entries; check exit 0, 1732 passed; fd704c42

### note · agent:manager-2 · 2026-10-16T04:17:49.228Z
integrated: 5c8980ccd3b63a81483bf23b574199836beb1d5a (branch bridle/b97n)

### note · agent:manager-2 · 2026-10-16T04:18:10.894Z
cleanup: removed agent b97n, branch bridle/b97n
