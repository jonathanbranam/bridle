---
id: b6mu
title: Self-upgrade drain may never restart when it starts during a spawn
kind: bug
opened: 2026-10-09
filed_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
needs: []
see: [h7gt]
tasks: [br-b6mu]
---

## The ask

Reproduce the spawn-in-flight vs. drain race (a test that starts the drain during the spawn,
with no go-file gate) and make sure the daemon either lets the held first prompt through or
stops counting the spawn, so the restart always comes. Restore test coverage of that race.

See br-h7gt (065b9443) and `docs/context/incidents.md` (2026-10-09 15:56).

## What happened

br-h7gt made `upgrade_test self_upgrade_restarts_only_after_the_mid_turn_agent_finishes`
deterministic by holding the build until the agent is seen working. The flake it fixed
(CI runs 37954619068, 37958674369, Linux) timed out after 60 s waiting for *either* the
agent to work *or* the restart to be requested. The old test already accepted the restart as
a pass, so on CI neither happened within 60 s.

## Suspected cause (unproven)

The drain began while the spawn was in flight and held the agent's first prompt. With no
init, the agent never reads working; if the quiet check still counts that spawn as busy, no
quiet point ever comes and the restart never happens. That is a daemon liveness bug, of the
same family as incident br-y455 (a stuck `spawning` flag meant no quiet point for ~23 h).
h7gt's worker also noted: lifting a drain after a failed restart (`perform_restart` error
path) doesn't deliver messages held during the drain.
