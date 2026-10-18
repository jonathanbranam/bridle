+++
id = "br-4dzj"
title = "link-check Stop hook links tickets to the session's project, not the ticket's (wrong links from a home-project orchestrator)"
kind = "bug"
state = "planned"
created_at = "2026-10-18T13:53:21.625Z"
updated_at = "2026-10-18T14:11:26.033052Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
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
