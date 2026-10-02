---
id: cy2v
title: "One watcher for every project: 'bridle agent wake --all-projects'"
kind: feature
opened: 2026-10-02
repos: [bridle]
changes: []
specs: []
needs: []
see: [interactive-sessions-context-for-all-daemon-decided-wakes-re-jttf, a-life-assistant-agent-on-the-notes-repo-phyy]
tasks: [br-1ddd]
---

## The ask

An orchestrator (or any principal) holding tokens for several projects needs one waiter per
daemon today: each project runs its own daemon (port, database, tokens), and
`bridle orchestrator wait-for-wake` is one long-poll against one daemon. Since 7b22c1e the
orchestrator role runs one waiter per project; miss one and that project's messages are
never seen.

The human (2026-10-01), on the choice between the CLI fanning out and daemons forwarding
wakes to each other: "Yes definitely agree - Recommended: the CLI watches every daemon at
once. bridle agent wake --all-projects".

## Decided

- **Client-side fan-out.** `bridle agent wake --all-projects` long-polls every project the
  caller holds a token for (its section in `~/.bridle/credentials.toml`, resolved through
  the registry as `--project` does) and returns on the first wake from any of them. No
  daemon-to-daemon forwarding.
- Each printed wake names its project, so the reader knows which daemon to act on.
- Wakes that arrive on other daemons while one returns stay queued there (the daemons already
  queue undelivered wakes), so exiting on the first one loses nothing.
- A project whose daemon is down or unreachable is reported once and skipped; it doesn't stop
  the others.

## Order

After `bridle agent wake` absorbs the orchestrator's wake reasons and replaces
`wait-for-wake` (jttf decision 2; parked for Saturday 2026-10-03 because it is the
orchestrator's own loop). Then the orchestrator role drops "one waiter per project" for this.

## Out of scope

Cross-machine projects (`[orchestrator.<machine>]` tokens) unless they come free; a
federated wake service.
