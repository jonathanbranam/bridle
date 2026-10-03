+++
id = "br-1ddd"
title = "One watcher for every project: 'bridle agent wake --all-projects'"
kind = "feature"
state = "planned"
created_at = "2026-10-02T00:21:16.323Z"
updated_at = "2026-10-02T00:22:10.586621Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: cy2v
Ticket: docs/tickets/open/one-watcher-for-every-project-bridle-agent-wake-all-projects-cy2v.md (the human's decisions are in it; read it all). Code: crates/bridle/src/commands/agent.rs ('bridle agent wake'), credentials and registry resolution (~/.bridle/credentials.toml; how --project resolves a daemon, crates/bridle-api discovery), the orchestrator role text workflow/base/roles/orchestrator.md. Depends on br-2672 (agent wake absorbs wait-for-wake) and br-a4a6.

Goal: 'bridle agent wake --all-projects [<identifier>]' long-polls every project the caller holds a token for and returns on the first wake from any. Client-side fan-out only (no daemon-to-daemon forwarding): one concurrent long-poll per project (tokio tasks), resolved like --project does; cancel the rest when one wakes. Each printed wake names its project (JSON: a 'project' field; text: a prefix). Wakes that arrive elsewhere stay queued on those daemons, so nothing is lost. A project whose daemon is down or unreachable is reported ONCE on stderr and skipped, and retried on the next call; it never stops the others. If no project is reachable, exit non-zero with the list. The identifier defaults to the caller's principal in each project.
Then change the orchestrator role text from 'one waiter per project' to this one command (and the doc in orchestrator-supervision.md/cli.md), CHANGELOG.
Tests: two fake daemons, wake on the second returns it named with its project; one daemon down is reported once and the other still works; all down fails; cancellation leaves no stray polls; the first wake wins when both fire.
Acceptance: just check passes. Model: Sonnet.
Migration plan: none (CLI behaviour and base role text; project-own orchestrator.md appends to base and inherits).
PARKED FOR SATURDAY: plan only now; the PM queues it after br-2672 lands, on or after Sat 2026-10-03 (the human: do not queue earlier). Out of scope: cross-machine projects unless free; a federated wake service.
