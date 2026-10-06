+++
id = "br-ubdc"
title = "bridle --project X send fails with an empty 'error: unknown:' when BRIDLE_PROJECT names a different project"
kind = "bug"
state = "pending"
created_at = "2026-10-06T00:08:25.489Z"
updated_at = "2026-10-06T00:08:25.489Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
+++

Repro (CLI 0.5.0, 2026-10-05 ~8 PM ET): with BRIDLE_PROJECT=bridle, 'bridle --project bridle-ui send <to> <body>' (or bridle send ...) prints 'error: unknown: ' and sends nothing. 'BRIDLE_PROJECT=bridle-ui bridle send ...' works. The reverse also fails (bridle-ui aide: --project bridle). Reads (inbox, task list, agents) with --project work. Suspect: send picks the token or principal from the env project but the URL from --project, and the error body is empty, so the CLI shows 'unknown'. Fix the mismatch and make the error say what failed.
