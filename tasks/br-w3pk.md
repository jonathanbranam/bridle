+++
id = "br-w3pk"
title = "Retire scripts/claude-orchestrator and claude-advisor: the orchestrator supervisor still relaunches through the wrapper"
kind = "chore"
state = "open"
created_at = "2026-10-06T23:08:51.212Z"
updated_at = "2026-10-06T23:09:22.000909Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: w3pk
docs/tickets/open/retire-scripts-claude-orchestrator-and-claude-advisor-the-or-w3pk.md

## Thread

### note · external:orchestrator · 2026-10-06T23:09:22.000Z
From orchestrator: br-w3pk is ready, a small cleanup the human raised. The daemon still relaunches the orchestrator through scripts/claude-orchestrator. It goes in two steps: change the relaunch line first, delete the scripts after the daemon upgrades. The brief is the ticket. Place it as normal work, not urgent.
