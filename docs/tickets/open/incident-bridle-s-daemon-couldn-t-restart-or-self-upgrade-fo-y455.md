---
id: y455
title: "Incident: bridle's daemon couldn't restart or self-upgrade for ~23 h: a stuck 'spawning' flag meant no quiet point ever came"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [2ax5, q7mv]
tasks: [br-y455]
---

## The ask

The human, 2026-10-06 ~6:20 PM ET (verbatim): "File an incident and schedule a post-mortem." About the orchestrator's finding below.

## What happened (as known)

- bridle's daemon (pid 23474, launchd dev.bridle.bridle) could not restart or self-upgrade from about 2026-10-05 23:01Z until the human ran `launchctl kickstart -k` at 2026-10-06 22:12Z.
- Every graceful restart failed after its wait: "not restarted: no quiet point within 600s; still busy: a spawning agent". `bridle agents` showed every agent idle. server.rs (restart path) waits on `state.manager.spawning()`, which stayed true with no agent spawning: a leaked flag or counter.
- Likely trigger, unconfirmed: worker a-897zk (incident-2y3m) was stop-requested 3 s after `agent.spawned` (2026-10-06 03:20:31-36Z).
- Earlier self-upgrade failures were reported as "no quiet point ... still busy: usage-history" (2026-10-05 23:49Z), so the trigger may be older.
- Consequences: the daemon stayed on pre-br-3haz code, so it had no /v1/outbox and cross-project messaging broke (incident br-2ax5). bridle's self-upgrade never ran. The orchestrator couldn't restart it; only the human could.

## The postmortem must answer

- The exact cause of the stuck flag: which spawn path doesn't release it (stop during spawn, spawn error, a panic?). Reproduce it in a test.
- Since when it was stuck: daemon logs and events for the first "a spawning agent" refusal; why nobody noticed failing self-upgrades for about 23 h.
- Why a failed self-upgrade (repeated "no quiet point") raises no alert to the orchestrator or the human.
- Why the restart error doesn't name what holds the flag.
- Fixes: release on every exit path, an alert after N refused upgrades (cf. q7mv's recommendations for incident aqa7, which weren't acted on), and a safe forced restart the orchestrator may run.

Related: br-btdn (the bug), br-2ax5, q7mv (aqa7 postmortem), br-4zfa.
