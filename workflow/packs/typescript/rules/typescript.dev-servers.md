---
id: typescript.dev-servers
severity: should
roles: [worker, reviewer]
---
Never kill or restart another agent's dev server; never take a port you
weren't given; stop what you start.

Tell your own processes apart by PID (keep a record and check it's still
running before killing it). If a port is unexpectedly in use, investigate
whether another agent owns it before taking action.

Why: agents work concurrently on the same machine and may share resources.
Unilaterally stopping others' processes breaks their work; owning what you
start and cleaning up ensures a stable shared environment.
