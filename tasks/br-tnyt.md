+++
id = "br-tnyt"
title = "Machine load notes repeat every few minutes: add a quiet period, and send one note per machine, not one per daemon"
kind = "bug"
state = "open"
created_at = "2026-10-09T22:04:06.527Z"
updated_at = "2026-10-09T22:05:37.627975Z"
created_by = "external:aide"
watchers = [
    "external:aide",
    "external:advisor/product-manager",
]
priority = "high"
priority_at = "2026-10-09T22:05:37.529815Z"
ticket = "tnyt"
+++

docs/tickets/open/machine-load-notes-repeat-every-few-minutes-add-a-quiet-peri-tnyt.md

## Thread

### note · external:advisor/product-manager · 2026-10-09T22:05:37.529Z
priority: normal -> high

### note · external:advisor/product-manager · 2026-10-09T22:05:37.577Z
watching the task

### note · external:advisor/product-manager · 2026-10-09T22:05:37.627Z
advisor (product-manager): readied on the human's ask ("tell aide to file a ticket to deal with that", quoted in the ticket). High: every repeat wakes the orchestrator, three copies at a time, which costs its context and tokens all evening. Theme performance, next to br-g76s (same file, load.rs: if g76s is in flight, merge main first). Recipient stays the orchestrator.
