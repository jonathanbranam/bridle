+++
id = "br-1ddd"
title = "One watcher for every project: 'bridle agent wake --all-projects'"
kind = "feature"
state = "dropped"
created_at = "2026-10-02T00:21:16.323Z"
updated_at = "2026-10-10T02:59:05.891803Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "external:advisor/product-manager",
]
ticket = "cy2v"
+++

Ticket: docs/tickets/open/one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v.md (the human's decisions are in it; read it all). Code: crates/bridle/src/commands/agent.rs ('bridle agent wake'), credentials and registry resolution (~/.bridle/credentials.toml; how --project resolves a daemon, crates/bridle-api discovery), the orchestrator role text workflow/base/roles/orchestrator.md. Depends on br-2672 (agent wake absorbs wait-for-wake) and br-a4a6.

Goal: 'bridle agent wake --all-projects [<identifier>]' long-polls every project the caller holds a token for and returns on the first wake from any. Client-side fan-out only (no daemon-to-daemon forwarding): one concurrent long-poll per project (tokio tasks), resolved like --project does; cancel the rest when one wakes. Each printed wake names its project (JSON: a 'project' field; text: a prefix). Wakes that arrive elsewhere stay queued on those daemons, so nothing is lost. A project whose daemon is down or unreachable is reported ONCE on stderr and skipped, and retried on the next call; it never stops the others. If no project is reachable, exit non-zero with the list. The identifier defaults to the caller's principal in each project.
Then change the orchestrator role text from 'one waiter per project' to this one command (and the doc in orchestrator-supervision.md/cli.md), CHANGELOG.
Tests: two fake daemons, wake on the second returns it named with its project; one daemon down is reported once and the other still works; all down fails; cancellation leaves no stray polls; the first wake wins when both fire.
Acceptance: just check passes. Model: Sonnet.
Migration plan: none (CLI behaviour and base role text; project-own orchestrator.md appends to base and inherits).
PARKED FOR SATURDAY: plan only now; the PM queues it after br-2672 lands, on or after Sat 2026-10-03 (the human: do not queue earlier). Out of scope: cross-machine projects unless free; a federated wake service.

## Thread

### note · agent:pm-1 · 2026-10-03T13:08:08.319Z
Held (orchestrator, via pm-1): waits until the human settles k8jn. Read-on-delivery can lose messages; the fan-out would drop a second daemon's reply after marking it read. Don't queue or start.

### note · external:advisor/product-manager · 2026-10-09T11:04:37.137Z
watching the task

### note · external:advisor/product-manager · 2026-10-10T02:59:05.891Z
dropped: the human, 2026-10-09 ~10:55 PM ET, chose the real fix over the stopgap: every wake is a message, one waiter per principal (br-n7cg then br-rhba); --all-projects would be a no-op once they land.
