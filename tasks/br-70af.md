+++
id = "br-70af"
title = "Incidents as a task kind: build kind, states and the broadcast notice"
kind = "feature"
state = "planned"
created_at = "2026-09-30T01:29:57.368Z"
updated_at = "2026-09-30T01:30:00.468623Z"
+++

GOAL: implement the design in docs/design/agent-host/incidents.md as revised by the preceding task (incident = a task kind owned by the orchestrator; potential -> active -> resolved; persistent broadcast notice to every agent while active, including agents that start, resume or renew; resolved note to those who saw it; undelivered notices dropped). Read that doc first, plus crates/bridle-daemon/src/tasks.rs (task state machine), the message queue and agent start/resume/renew paths in crates/bridle-daemon, crates/bridle-api/src/types.rs (wire contract: change clients and daemon together), and crates/bridle/src/cli.rs. Acceptance: just check passes; tests for filing a potential incident, promote/drop/resolve permissions (orchestrator/human only), notice delivered to a running agent, to an agent started while active, and dropped-if-undelivered on resolve; docs in step (cli.md, storage.md, CHANGELOG). Model: Sonnet. Out of scope: cross-project incidents, daemon-detected incidents, budget holds becoming incidents, TUI display. Touches the task code and types.rs like br-c83e (human to-dos): run after it, not in parallel.
