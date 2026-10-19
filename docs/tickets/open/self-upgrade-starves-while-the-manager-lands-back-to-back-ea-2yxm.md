---
id: br-2yxm
title: "Self-upgrade starves while the manager lands back to back: each land's --postpone 45m pushes the upgrade out again"
kind: bug
created: 2026-10-19
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Seen 2026-10-19 ~7-8 PM ET (orchestrator): a green-CI self-upgrade was postponed for over an hour (19:08, 19:27, 19:46, 20:06). manager-2 runs `bridle daemon restart --postpone 45m` before every land (its handover routine) and lands came every ~20 min, so each land pushed the upgrade out again. A human was waiting on a landed fix (br-ggbx) going live.

Fix (pick the smaller): a postpone covers only the land in progress and ends when the land ends; or a pending upgrade gets a window between lands (no new claim or land starts until it runs).

Files likely: the daemon's self-upgrade/postpone code (crates/bridle-daemon), `bridle daemon restart` in crates/bridle, the manager's land routine text in workflow/ if it asks for the postpone. Docs: docs/design/agent-host/ (self-upgrade section) kept in step.

Check: `just check`, plus a test that two back-to-back postponed lands do not delay a pending upgrade past the second land's end.

Model: Sonnet. Out of scope: changing the upgrade cadence or CI gating.
