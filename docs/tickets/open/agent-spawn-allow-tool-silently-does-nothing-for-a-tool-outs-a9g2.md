---
id: a9g2
title: agent spawn --allow-tool silently does nothing for a tool outside the role's --tools set (WebSearch on a worker)
kind: bug
opened: 2026-10-07
repos: [bridle]
changes: []
specs: []
needs: []
see: []
tasks: [br-a9g2]
---

## The ask

Seen 2026-10-06 on bridle-ui's ui-m2pz. manager-1 spawned worker picker-proto with `--allow-tool WebSearch --allow-tool WebFetch` for a research step. The worker reported that neither tool was offered to it, so the research went unsourced (rule report-task-failures, reported to the human).

Cause: `--allow-tool` (`extra_allowed_tools`) is only added to `--allowedTools`, which grants permission (`crates/bridle-daemon/src/supervisor.rs`, the spawn and resume paths). The role's `tools` (`--tools`, the lean-context tool set, spike 08) is passed unchanged. For the built-in worker it leaves out WebSearch and WebFetch, so their definitions never reach the agent. The spawn succeeds and nothing warns.

The `researcher` built-in role already has the web tools. That is the right choice here, and manager-1 has been told.

Fix (pick the smaller one): an `--allow-tool` naming a built-in tool missing from the role's `tools` is also added to `--tools`; or the spawn is refused with "<tool> isn't in the <role> role's tool set; use the researcher role". Say which in `docs/design/cli.md` (`agent spawn`). Check: a supervisor test with the fake claude shows the argv; `just check`.
