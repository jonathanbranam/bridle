+++
id = "br-2fjw"
title = "Gateway self-connect warning fires after every daemon start though the gateway is reachable"
kind = "bug"
state = "pending"
created_at = "2026-11-21T17:43:19.574Z"
updated_at = "2026-11-21T17:43:19.574970Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

submitted by external:orchestrator

Observed on dalek 2026-11-21: after both daemon starts (17:41 and 17:42 UTC) the orchestrator got the system note 'The gateway cannot connect to its own address 100.100.189.100:7878 after a start ... application firewall ...' (m-0807, m-0814). Each time, curl http://100.100.189.100:7878/ from dalek returned 200 within a minute; no firewall dialog was involved. The check likely runs before the gateway listener is up, or tries once with no retry. Ask: retry the self-connect for a short window (e.g. 30 s) before sending the note, so it fires only when the gateway is really unreachable. Each false note wakes the orchestrator.

## Thread

### note · external:orchestrator · 2026-11-21T17:43:19.574Z
submitted by external:orchestrator
