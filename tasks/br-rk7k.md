+++
id = "br-rk7k"
title = "bridle-ui: a 'Send to an agent' button on a task: pick the agent, type a message, send"
kind = "feature"
state = "integrated"
created_at = "2026-10-05T00:09:25.550Z"
updated_at = "2026-10-05T09:53:19.419958Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:aide",
]
branch = "bridle/send-to-agent"
commit = "811c626d1d55287c87b65522f8f355db36c10fc3"
summary = "Gateway write route POST /api/v1/projects/{project}/messages {to, text, task?} and GET .../recipients (crates/bridle-gateway/src/messages.rs). Sends via the daemon's POST /v1/messages (as bridle send --task does) with the human token, so sender is human; recipients are the project's running agents plus 'orchestrator' (external:orchestrator); others get 422 (new ActionError::Invalid), empty text 400, text capped at 8000 chars, unknown project 404. Types in bindings/. human-web-ui.md section 2 records the loosening (message only, no start/stop/kill/events). One project per request; no UI."
+++

original id: rk7k
Bridle-repo (gateway) half of docs/tickets/open/bridle-ui-a-send-to-an-agent-button-on-a-task-pick-the-agent-rk7k.md (read it; it quotes the human). The UI half is filed by the orchestrator after this is planned. Goal: a gateway WRITE route that sends a message from 'human' to one agent, in crates/bridle-gateway beside the read-only tasks/agents routes of br-s6cj and br-7sd9 (those must land first; they supply the agent list and the same route registration and doc: dependency edges on both). Shape: POST /api/v1/projects/{project}/messages (name per the existing route style) with { to, text, task? } ; the gateway proxies the daemon's existing agent-message endpoint (/v1/agents/{id}/messages, same as bridle send) with the gateway's per-project human credential so the sender is recorded as human; optional task id is added to the message so it threads on the task (check how bridle send --task does it and use the same). Recipients: only an agent or the orchestrator that the project's daemon lists; reject others (404/422), cap the text length, empty text refused. For the dropdown, GET agents already exists (br-7sd9); add the orchestrator to the recipient list in a small GET .../recipients route if the agents list lacks it. Doc: docs/design/human-web-ui.md section 2: record that the human's ask loosens 'no agent control' for this ONE write (send a message), still no start/stop/kill and no events, and why (a stolen session can message, not run work directly; messages are visible in the audit trail). Cross-project look is open (the human: by default the task's project's agents plus the orchestrator): build for that default only, one project per request. Tests: send ok with sender human, unknown recipient refused, empty text refused, unknown project 404. Acceptance: just check passes. Model: Sonnet. Out of scope: any UI, events, other writes, cross-project pickers.

## Thread

### note · external:orchestrator · 2026-10-05T00:09:37.921Z
From orchestrator: br-rk7k (ticket rk7k, the human's ask via aide) is open: a 'Send to an agent' button on a task in bridle-ui. Plan the bridle half after br-s6cj and br-7sd9 (it needs the Tasks page and agent list): a gateway write route that sends a message from 'human' to an agent, plus the human-web-ui.md change (the human's ask loosens 'no agent control' for this one write; record it there). The cross-project look is open: default recipients are the task's project's agents plus the orchestrator. Sonnet. A bridle-ui half follows, which I'll file when the gateway route is planned.

### note · external:aide · 2026-10-05T00:09:45.622Z
watching the task

### note · agent:send-to-agent · 2026-10-05T09:33:59.815Z
done: gateway POST .../messages + GET .../recipients, 6 tests, docs and CHANGELOG; just check exit 0, 1220 tests passed; dc5316bb

### note · agent:send-to-agent · 2026-10-05T09:42:03.478Z
update: merged main (br-a3yd) cleanly, no conflicts; just check exit 0, 1221 tests passed; new tip b57888fc (use this for --checked-commit)

### note · agent:manager-2 · 2026-10-05T09:53:19.419Z
integrated: 811c626d1d55287c87b65522f8f355db36c10fc3 (branch bridle/send-to-agent)
