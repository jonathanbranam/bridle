+++
id = "br-p9sq"
title = "Before br-q7my reaches your machines: check each machine's doctor, then let it self-upgrade (dalek, vesta, remus)"
kind = "chore"
state = "claimed"
created_at = "2026-10-25T16:07:02.619Z"
updated_at = "2026-10-25T16:08:45.870628Z"
created_by = "agent:manager-2"
watchers = [
    "agent:manager-2",
    "human",
]
priority = "high"
priority_at = "2026-10-25T16:07:02.621321Z"
ticket = "p9sq"
+++

docs/tickets/open/before-br-q7my-reaches-your-machines-check-each-machine-s-do-p9sq.md

## Thread

### note · agent:manager-2 · 2026-10-25T16:07:02.621Z
created for the human, priority high

### note · agent:manager-2 · 2026-10-25T16:07:02.623Z
To-do for you (high priority): Before br-q7my reaches your machines: check each machine's doctor, then let it self-upgrade (dalek, vesta, remus). Finish it with `bridle ticket done br-p9sq`.

### note · external:orchestrator · 2026-10-25T16:07:52.595Z
orchestrator: step 1 on dalek done: of the 9 dalek projects only dalek-home has [orchestrator] (enabled = true); [machine] name = "dalek"; doctor 'orchestrator home' ok. Asked orchestrator@vesta and @remus to run the same check; I will tell manager-2 to push when both report clean.

### note · external:orchestrator · 2026-10-25T16:08:33.610Z
orchestrator: step 1 on remus clean (orchestrator@remus, m-0455): remus-home, data-contracts (old clone) and dotfiles/local-df have no [orchestrator] section. remus-home lacking [orchestrator] does not trip q7my: MachineHome::problem refuses only on a missing [machine] name, a missing home project, or a project != <name>-home (checked in 4f23dcd1). Separate, pre-existing: without [orchestrator] enabled the remus-home daemon does not supervise/relaunch remus's orchestrator. Waiting on vesta.

### note · external:orchestrator · 2026-10-25T16:08:45.870Z
orchestrator: step 1 done on all three. vesta (orchestrator@vesta, m-0456): vesta-home enabled = true; meta-notes enabled = false; notes/dotfiles-local/meta-notes-ui none. One exception: the vesta bridle clone (/srv/shared/work/bridle-work/bridle) has [orchestrator] enabled = true but runs no daemon on vesta, so nothing is supervised; if a daemon were ever started there, q7my makes it log an error and start no orchestrator. Harmless; optional cleanup. Told manager-2 to push. Steps 2-3 (self-upgrade, then 'bridle up' says already running) remain for the human.
