---
id: 2msq
title: bridle send --project to a local daemon needs a peer token, and --url can't be combined with --project
kind: bug
opened: 2026-10-10
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask

Seen by the orchestrator, 2026-10-10 ~02:32Z, onboarding data-contracts (m5n2). The orchestrator holds an orchestrator token for data-contracts (local daemon, port 7407). `bridle task ready/comment --project data-contracts ...` work, but:

- `bridle send --project data-contracts manager "..."` fails: "no peer token for 'data-contracts': its human runs `bridle token create --peer dalek` there ...". Send routes through mail between daemons even when the caller holds a direct token for that project on this machine.
- `bridle send --url http://127.0.0.1:7407 --project data-contracts manager ...` fails: "$BRIDLE_AS=orchestrator but the project is unknown (the daemon was found by URL): pass --project, or set $BRIDLE_TOKEN", although --project was passed.

Expected: with a direct token for the project, send talks to that daemon directly like the other commands; and --project names the token when --url picks the daemon.

Impact: low tonight (the manager picked the task up from its own "task filed" note), but a new project's orchestrator cannot message its agents until the human makes a peer token. Filed by orchestrator; scheduling is the PdM's.
