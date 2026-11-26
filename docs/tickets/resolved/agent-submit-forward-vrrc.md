---
id: br-vrrc
title: "Agents can't submit tickets to another project: forward ticket submit through the agent's own daemon"
kind: fix
created: 2026-11-26
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-5akm]
resolved: 2026-11-26T18:08:06Z
---

## The ask

An agent can file a ticket on another project, as rule `file-tickets-on-other-projects` says (`bridle ticket submit --project <project>`). Today it errors.

Found by manager-2 when br-5akm landed (1bfaf4b8): br-5akm made `submit --project <other>` fail loudly instead of silently filing on the agent's own daemon, because an agent session has no registry or token for other daemons. Until this lands, agents ask the orchestrator to submit.

## The work

Forward the submission through the agent's own daemon to the target project's daemon, as mail between projects is forwarded (find that path: the outbox/relay code in `crates/bridle-daemon`, and br-5akm's summary with `bridle ticket show br-5akm`). The agent never gets another daemon's token. The task on the target keeps the real submitter (`agent:<name>@<project or host>` as mail does) so a drop's reason gets back to them. A target that is unreachable errors, or queues and says so; never silently dropped, never filed locally. Check who may submit across projects follows today's rules for cross-project messages; do not widen it. Update `docs/design/agent-host/` and `docs/design/cli.md`; the rule text needs no change if the command now works.

Migration: none expected; say what an older target daemon does (clear error).

## Acceptance

`just check`; a test with two daemons: an agent-token submit with `--project B` creates the task on B with the right submitter, and a drop reason reaches the agent; an unreachable B reports it.

## Model

Sonnet. Out of scope: forwarding other ticket commands (comment, show) across projects.
