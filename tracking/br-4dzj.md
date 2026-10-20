+++
id = "br-4dzj"
title = "link-check Stop hook links tickets to the session's project, not the ticket's (wrong links from a home-project orchestrator)"
kind = "bug"
state = "integrated"
created_at = "2026-10-18T13:53:21.625Z"
updated_at = "2026-10-20T02:25:36.823445Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:w4dzj",
]
branch = "bridle/w4dzj"
commit = "5c57e7b3c5d7006fed465d089534289dc6d07417"
summary = "link-check hook now resolves each id to its owning project: link_check::owning (qualified id by prefix, bare id by first project having it, session's first, else session's), with other projects read from the machine registry (prefix + repo). unlinked_ids takes all known prefixes. Unit tests for the three cases. Registry is local-machine only."
ticket = "4dzj"
+++

docs/tickets/open/link-check-stop-hook-links-tickets-to-the-session-s-project-4dzj.md

submitted by external:orchestrator

Seen 2026-10-18 by the orchestrator, which now runs in dalek-home (br-7ytg). The `bridle hook link-check` Stop hook flags bare bridle ticket ids (gb97, z485, 7ytg, uysm, q7my) and suggests links like http://dalek.tailbc91f5.ts.net:7878/p/dalek-home/tickets/da-gb97. Those tickets are bridle's (br-gb97), so the suggested links are wrong. `bridle link br-gb97 --project bridle` gives the right one.

## The ask
The hook should resolve an id to the project that owns it: honour a qualified id's prefix (br-, da-), and for a bare id prefer the project where it exists, rather than always the session's own project. With every machine's orchestrator moving to its home project (7ytg), this now fires on almost every reply about bridle work.

Verify: in a dalek-home orchestrator session, a reply naming a bridle ticket either passes with a br- link or is told the /p/bridle/ link.

## Thread

### note · external:orchestrator · 2026-10-18T13:53:21.656Z
submitted by external:orchestrator

### note · agent:w4dzj · 2026-10-20T02:25:16.040Z
built: 55c020fc0b4f4d58e2fd94ceb8496c505ab9f454 (branch bridle/w4dzj)

### note · agent:w4dzj · 2026-10-20T02:25:16.559Z
done: link-check resolves owning project; just check exit 0, 1913 tests passed; 55c020fc0b4f4d58e2fd94ceb8496c505ab9f454

### note · agent:manager-2 · 2026-10-20T02:25:23.992Z
integrated: 5c57e7b3c5d7006fed465d089534289dc6d07417 (branch bridle/w4dzj)

### note · agent:manager-2 · 2026-10-20T02:25:36.823Z
cleanup: removed agent w4dzj, branch bridle/w4dzj
