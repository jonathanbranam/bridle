+++
id = "br-ubdc"
title = "bridle --project X send fails with an empty 'error: unknown:' when BRIDLE_PROJECT names a different project"
kind = "bug"
state = "pending"
created_at = "2026-10-06T00:08:25.489Z"
updated_at = "2026-10-06T21:27:01.296768Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
size = "S"
+++

Repro (CLI 0.5.0, 2026-10-05 ~8 PM ET): with BRIDLE_PROJECT=bridle, 'bridle --project bridle-ui send <to> <body>' (or bridle send ...) prints 'error: unknown: ' and sends nothing. 'BRIDLE_PROJECT=bridle-ui bridle send ...' works. The reverse also fails (bridle-ui aide: --project bridle). Reads (inbox, task list, agents) with --project work. Suspect: send picks the token or principal from the env project but the URL from --project, and the error body is empty, so the CLI shows 'unknown'. Fix the mismatch and make the error say what failed.

## Thread

### note · external:orchestrator · 2026-10-06T21:27:01.296Z
Root cause (orchestrator, 2026-10-06 5:30 PM ET): a regression from br-3haz (29296901, landed 10-05 7:02 PM; CLI installed 7:39 PM). Since then 'bridle send --project <other>' no longer sends direct: it POSTs to the sender's OWN daemon's /v1/outbox to forward. (1) bridle's daemon still runs pre-3haz code (its self-upgrade to 29296901 was deferred, 'no quiet point'), so /v1/outbox returns 404 with an empty body, which the CLI shows as 'error: unknown: '. Probed: POST 127.0.0.1:7401/v1/outbox -> 404. (2) Even on the new daemon, forwarding needs a [peer] token in credentials.toml, and there are none. Messages would queue and never arrive. So every cross-project send (aides, orchestrator) has failed for ~22 h. Same-project sends work. Fix: fall back to the direct send when the own daemon has no outbox or no peer token for the destination (same machine, the registry has its URL), and make a 404 or empty error body say what failed.
