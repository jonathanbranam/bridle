+++
id = "br-rztb"
title = "Incident: something keeps restarting dalek's gateway outside launchd from a Claude session (pid 88281 since 10-08 9:39 PM); recurring"
kind = "incident"
state = "open"
created_at = "2026-10-09T19:18:13.267Z"
updated_at = "2026-10-09T19:18:49.191262Z"
created_by = "external:aide"
watchers = ["external:aide"]
priority = "high"
priority_at = "2026-10-09T19:18:48.930751Z"
ticket = "rztb"
+++

docs/tickets/open/incident-something-keeps-restarting-dalek-s-gateway-outside-rztb.md

## Thread

### note · external:orchestrator · 2026-10-09T19:18:48.930Z
priority: normal -> high

### note · external:orchestrator · 2026-10-09T19:18:49.040Z
orchestrator: readied, high (the human, via aide, verbatim on the ticket: 'This needs to be investigated; I have reported this before'). Investigation first: find the caller, check whether any test (br-57nt's included) touches the real gateway or launchd. Step 4 (stopping pid 88281) needs the human's OK. Logged in docs/context/incidents.md.

### note · external:orchestrator · 2026-10-09T19:18:49.191Z
From orchestrator: br-rztb (gateway restarted outside launchd, recurring, human-reported) is ready, high. Plan it next after work already claimed; Sonnet. See thread.
