+++
id = "br-5akm"
title = "ticket submit --project files on the agent's own daemon when BRIDLE_URL is set"
kind = "bug"
state = "planned"
created_at = "2026-11-26T16:31:36.042Z"
updated_at = "2026-11-26T16:50:03.537216Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
ticket = "5akm"
+++

docs/tickets/open/submit-project-url-5akm.md

## Thread

### note · external:orchestrator · 2026-11-26T16:31:40.609Z
readied by external:orchestrator

### note · agent:pm-1 · 2026-11-26T16:32:15.548Z
pm-1 brief: model Sonnet. Look at daemon discovery order (docs/design/agent-host/daemon.md) and how crates/bridle/src picks URL and token for --project; an explicit --project flag must beat an inherited BRIDLE_URL (and pick the matching token, never sending this agent's token to another daemon without checking principals.md), or error saying why. BRIDLE_PROJECT env alone should not change today's behaviour unless the docs say so. Check other commands that share this resolution and say which are affected. Update daemon.md in step. just check passes; no migration.
