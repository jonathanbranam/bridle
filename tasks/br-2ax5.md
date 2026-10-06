+++
id = "br-2ax5"
title = "Incident: br-3haz broke every cross-project message for ~22 h: the CLI needed an outbox no running daemon had, and peer tokens nobody had made"
kind = "incident"
state = "planned"
created_at = "2026-10-06T21:40:37.479Z"
updated_at = "2026-10-06T21:40:49.046757Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: 2ax5
docs/tickets/open/incident-br-3haz-broke-every-cross-project-message-for-22-h-2ax5.md

## Thread

### note · agent:manager-2 · 2026-10-06T21:40:49.046Z
manager-2 input for the postmortem: I landed br-3haz slice 1 (br-3haz, 2026-10-05 ~23:02 UTC). I checked the diff, summary and a green just check, but not compatibility with already-running daemons (no running daemon had /v1/outbox) or missing peer tokens; the brief did not ask for either and I did not add the question. From ~00:49 UTC 10-06 I saw 'bridle send' fail with a bare 'error: unknown:' repeatedly (to the usage-history worker and to the orchestrator) and guessed 'likely br-ubdc' in my handover; I never tested a same-project vs cross-project send to isolate it, and never reported the failure as a regression. I could not tell anyone because the failing path was the messaging itself. Suggested gate for managers: a landing checklist item 'does this change a wire/CLI path that running daemons or other projects use? what is the migration?'.
