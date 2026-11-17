+++
id = "br-d4bz"
title = "daemon restart reports 'did not come back within 95s' when it came back in seconds"
kind = "bug"
state = "pending"
created_at = "2026-11-17T13:38:37.828Z"
updated_at = "2026-11-17T13:38:51.730137Z"
created_by = "external:orchestrator@remus"
watchers = ["external:orchestrator@remus"]
ticket = "d4bz"
+++

docs/tickets/open/daemon-restart-reports-did-not-come-back-within-95s-when-it-d4bz.md

submitted by external:orchestrator@remus

Seen twice on remus with the flume daemon (127.0.0.1:7412, also bound to its tailnet address): 2026-11-17 04:40 UTC and 13:36 UTC. 'BRIDLE_PROJECT=flume bridle daemon restart' printed 'restarting at <sha>: 1 agent to resume' and then 'error: the daemon did not come back within 95s; see .../daemon.log'. The log shows it back within 2 s (13:36:37 restart requested, 13:36:39 state branch at start), and 'bridle status' worked right away. Guess: the CLI polls an address or identity that doesn't match after the restart (the daemon listens beyond loopback). Impact: a false failure leads an operator to dig into a healthy daemon or restart it again.

## Thread

### note · external:orchestrator@remus · 2026-11-17T13:38:37.829Z
submitted by external:orchestrator@remus
