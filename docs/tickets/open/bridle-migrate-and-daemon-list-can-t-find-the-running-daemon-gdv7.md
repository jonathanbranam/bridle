---
id: br-gdv7
title: "bridle migrate (and daemon list) can't find the running daemon from agent sessions: 'no running daemon registered'"
kind: bug
created: 2026-10-31
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Seen on dalek-home by the orchestrator, 2026-10-31 ~6:05-6:16 AM ET, daemon 0.7.6 (manager message m-0548 on dalek-home). Ranks high: it blocks every project's opt-in migrations ticket (bridle files one per project on upgrade, e.g. track-web tw-yxux), and those are part of onboarding the human's real projects.

Opt-in migrations ticket da-t26q: the manager and its worker both ran `bridle migrate --only 0002-standard-ticket-fields --dry-run` and got `no running daemon registered for project dalek-home`, from the worker's worktree and the manager's clone. `bridle daemon list` in those sessions says `no daemons running`, yet `bridle status` reaches the daemon fine. The same dry run from the orchestrator's session (outside the agent sandbox, same clone) works.

Likely cause: the daemon registry (`~/.bridle/daemons`) is not visible to agents (sandbox, or agents' BRIDLE_HOME/HOME differs: see br-bfrz, where agents' BRIDLE_HOME has no config.toml), while the status path finds the daemon another way (BRIDLE_URL in the agent env).

Fix: make `bridle migrate` and `bridle daemon list` find the daemon from inside an agent session the way `bridle status` does (BRIDLE_URL / the discovery order in docs/design/agent-host/daemon.md), or let agents read the registry. Check the other commands that use the registry directly (grep for the "no running daemon registered" error) and fix them the same way. Read the discovery order doc before choosing; prefer reusing the existing discovery function over a new path.

Acceptance: a test where the registry is absent but BRIDLE_URL (and token) is set: `bridle migrate --dry-run` and `bridle daemon list` reach that daemon; from a spawned worker, `bridle migrate --dry-run` lists what it would apply; `just check`. Model: Sonnet. Out of scope: letting agents apply migrations they were not asked to (the migrate permissions stay as they are).
