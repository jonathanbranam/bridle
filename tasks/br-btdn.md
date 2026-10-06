+++
id = "br-btdn"
title = "Daemon restart and self-upgrade blocked forever: manager.spawning() stays true with no agent spawning ('still busy: a spawning agent')"
kind = "bug"
state = "pending"
created_at = "2026-10-06T21:56:19.445Z"
updated_at = "2026-10-06T21:56:19.445Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

Seen 2026-10-06 ~5:50 PM ET on bridle's daemon (pid 23474, running pre-br-3haz code since 2026-10-05 23:01Z). 'bridle daemon restart' times out after 600 s with 'no quiet point ... still busy: a spawning agent', while 'bridle agents' shows every agent idle. server.rs:3653 waits on state.manager.spawning(), which must be stuck true (a leaked spawning flag or counter). A likely trigger: worker a-897zk (incident-2y3m) was stop-requested 3 s after agent.spawned (2026-10-06 03:20:31-36Z). This is also why bridle's self-upgrade never ran (earlier 'no quiet point' failures), which left the daemon without br-3haz's /v1/outbox (incident br-2ax5). Fix: release the flag on every spawn exit path (stop during spawn, spawn error); log or expose what holds it; make the restart error name the agent.
