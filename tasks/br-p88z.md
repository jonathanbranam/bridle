+++
id = "br-p88z"
title = "Sign bridle with a stable local certificate on Macs, so the firewall's Allow survives every rebuild"
kind = "feature"
state = "open"
created_at = "2026-10-04T13:18:34.357Z"
updated_at = "2026-10-04T22:05:45.810783Z"
created_by = "external:advisor"
watchers = [
    "external:advisor",
    "external:aide",
]
priority = "high"
+++

original id: p88z
docs/tickets/open/sign-bridle-with-a-stable-local-certificate-on-macs-so-the-f-p88z.md

## Thread

### note · external:aide · 2026-10-04T22:00:06.237Z
watching the task

### note · external:aide · 2026-10-04T22:00:06.255Z
The human, verbatim (2026-10-04 ~6 PM ET, to the bridle-ui aide, after the gateway stopped answering from other machines following the 5:09 PM rebuild): "we've just got to fix this signing thing. That's just critical. We just got to get it fixed. I don't know where that... We've got a ticket. I don't know where the work on that is or why it isn't been prioritized. Um, it's a total pain in the ass, and it's happening constantly. So... Just uh, let's get that, that work moving."

### note · external:orchestrator · 2026-10-04T22:00:21.907Z
priority: normal -> high

### note · external:orchestrator · 2026-10-04T22:00:21.943Z
Orchestrator: critical per the human (quote above). Raised to high and sent to manager-2 to start now as a third worker, ahead of bek3/tc7t. Constraint from aide: the human has SSH only tonight, so the one-time cert setup must work over SSH with the security CLI (no keychain dialog), or wait for them at dalek; the code half (sign with the identity if present, ad-hoc otherwise) doesn't need them.

### note · external:aide · 2026-10-04T22:05:45.810Z
Confirmed 2026-10-04 evening: the gateway stopped answering from other machines after the 5:09 PM ad-hoc rebuild (it still answered locally, and the firewall listed bridle as allowed). Re-registering the binary over SSH (socketfilterfw --remove / --add / --unblockapp) fixed it. The human: "That last set of three pseudos fixed it."
