+++
id = "br-2ax5"
title = "Incident: br-3haz broke every cross-project message for ~22 h: the CLI needed an outbox no running daemon had, and peer tokens nobody had made"
kind = "incident"
state = "planned"
created_at = "2026-10-06T21:40:37.479Z"
updated_at = "2026-10-06T22:20:58.226835Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 2ax5
docs/tickets/open/incident-br-3haz-broke-every-cross-project-message-for-22-h-2ax5.md

## Thread

### note · agent:manager-2 · 2026-10-06T21:40:49.046Z
manager-2 input for the postmortem: I landed br-3haz slice 1 (br-3haz, 2026-10-05 ~23:02 UTC). I checked the diff, summary and a green just check, but not compatibility with already-running daemons (no running daemon had /v1/outbox) or missing peer tokens; the brief did not ask for either and I did not add the question. From ~00:49 UTC 10-06 I saw 'bridle send' fail with a bare 'error: unknown:' repeatedly (to the usage-history worker and to the orchestrator) and guessed 'likely br-ubdc' in my handover; I never tested a same-project vs cross-project send to isolate it, and never reported the failure as a regression. I could not tell anyone because the failing path was the messaging itself. Suggested gate for managers: a landing checklist item 'does this change a wire/CLI path that running daemons or other projects use? what is the migration?'.

### note · external:orchestrator · 2026-10-06T22:20:58.226Z
How br-3haz was tested (checked 2026-10-06 ~6:45 PM ET): crates/bridle-daemon/tests/outbox_test.rs does start two in-process daemons (alpha, beta), mints a peer token on beta, writes it under [peer] and forwards. 6 tests: queue then deliver, destination down, refused-for-good doesn't block, repeated forward delivered once, only a peer token is believed, no peer token refused. All through the API client, so the daemons were tested together. Gaps: (1) the CLI path 'bridle send --project <other>' was never run against two daemons; project_resolution_test only changed its expectation to 'hits own daemon'. (2) No new-CLI-against-old-daemon test, and every running daemon was old. (3) No CLI test with no [peer] token: the daemon refuses it up front, but the CLI printed an empty 'error: unknown:'. (4) No test of agent sends inside one project. In bridle-ui, on the old daemon, every agent send failed, even to its own manager. (5) No test that a queued message is ever retried: per the commit, it's retried only on the next send to the same destination. The human's test message o-0002 (bridle-ui -> bridle, 22:2xZ) is still undelivered. The human: it should have been tested by starting two bridle daemons, minting tokens and passing calls between them, end to end through the CLI as a user would.
