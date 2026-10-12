---
id: br-3pc2
title: "Inventory: everything a project has on a machine, and what moves on a transfer"
kind: research
created: 2026-10-12
created_by: external:advisor/product-manager
theme: multi-machine
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-kx5u, br-kt25, br-53gy, br-rjd5]
parent: br-kx5u
---

The human, 2026-10-11 ~8:47 PM ET (to advisor product-manager), verbatim: "I will test with a smaller proejct firs, but this needs to work; also, i'm concerned about everything that is not moving oer". Part of epic br-kx5u (set up a new machine, transfer a project).

## The ask

A read-only inventory, from the code and docs (not memory), of everything a bridle project has on the machine that serves it, and for each item: where it lives, whether it moves on a transfer today, and how it should (carry, recreate, or name as left behind).

Start from the seven places in br-rjd5 and the gaps the PdM found tonight (br-53gy thread): `<workspace>/.bridle/bridle.db` (principals and every token, messages and inboxes, outbox, schedules, agents, sessions, costs), the state branch, the integration branch and unpushed commits, worktrees and uncommitted work, Claude Code session transcripts (resume), `~/.bridle/config.toml` (`[projects]`, `[machines]`, `[machine]`, mail, gateway), `credentials.toml` (role, `<role>.<machine>`, `[peer]`), `.bridle/tokens/human`, `~/.bridle/daemons/<p>.json`, the launchd plist or systemd unit, the mail bridge's project list, the gateway, `owner.toml` and the pre-push hook, logs, benchmarks, and anything else found. Also: what is special about moving bridle itself (the machine's orchestrator runs in it).

Deliverable: a table in this ticket (item, location, moves today?, proposal), plus a short list of what kt25 and 53gy must add. No code changes.
