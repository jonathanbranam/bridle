+++
id = "br-z7y5"
title = "Incident: syspolicyd and Spotlight pegged, builds and app launches stalled: each spawn clones a 564K-file integration target/"
kind = "incident"
state = "pending"
created_at = "2026-10-06T23:32:04.715Z"
updated_at = "2026-10-07T00:12:06.444534Z"
created_by = "external:orchestrator"
watchers = ["external:orchestrator"]
+++

original id: z7y5
docs/tickets/open/incident-syspolicyd-and-spotlight-pegged-builds-and-app-laun-z7y5.md

## Thread

### note · external:orchestrator · 2026-10-06T23:41:11.211Z
orchestrator: remediation 2026-10-06 ~7:40 PM ET. The human killed the target copy (pid 88006) and deleted integration/target, and excluded the work volume from Spotlight (Spotlight privacy policy). Config 2f6860c0 turns off [worktrees] warm_target and [integration] warm_build (the human's edit); a daemon restart was requested to load it. Still open: the iTerm Developer Tools exemption (the human's), and a code fix or removal of the warm build.

### note · external:orchestrator · 2026-10-07T00:12:06.444Z
orchestrator: aftermath. The br-qbbk integration check has hung for ~60 min in gateway_test a_replaced_binary_is_re_executed: its re-executed 'bridle gateway' copy (pid 25731) sits at 0% CPU, probably stuck behind syspolicyd when it launched. It blocks every landing, the critical br-x56y included. The test has no timeout of its own, and nextest didn't end it.
