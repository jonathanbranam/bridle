+++
id = "br-m5n2"
title = "Onboard data-contracts: start the trial that was prepared on 2026-09-28"
kind = "chore"
state = "open"
created_at = "2026-10-10T02:18:06.360Z"
updated_at = "2026-10-10T02:23:32.195432Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "m5n2"
+++

docs/tickets/open/onboard-data-contracts-start-the-trial-that-was-prepared-on-m5n2.md

## Thread

### note · external:advisor/product-manager · 2026-10-10T02:18:15.772Z
advisor (product-manager): readied on the human's ask (quoted in the ticket). For the orchestrator: steps 1-3 of the ticket (refresh bridle-adopt's config, start the daemon under launchd, file the dc- tasks) need no worker slot and may run tonight; step 4 (vpaw, a worker) only once the machine-setup checks (br-88d4, br-751e, br-fpde) are through or a slot frees without delaying them. data-contracts main is never touched. The four ADOPT-REVIEW questions go ahead on the recommended answers; the human may amend in the morning.

### note · external:advisor/product-manager · 2026-10-10T02:18:26.758Z
advisor (product-manager): from the aide's check (m-8799): follow docs/context/adding-a-project.md. Also left: a [projects] line in ~/.bridle/config.toml (free port on dalek: 7407); tokens (bridle token create) are the human's to run in a plain terminal, so that step waits for the morning. The human, via aide: "there is already some setup ... and a bridle-adopt branch; i'm not sure what is left undone" (the ticket's 'Where it stands' answers that). Tonight: config refresh and daemon setup only if they need no worker; no dc worker tonight.

### note · external:orchestrator · 2026-10-10T02:20:44.627Z
orchestrator: steps 1-2 done. bridle-adopt 09b85ff pushed (config already matched track-web; ADOPT-REVIEW Q2 resolved: sync now writes arch-guard/kill-guard hooks, settings.json committed; .gitignore runtime entries; ADOPT-REVIEW start-up steps updated for launchd on 7407). data-contracts main untouched; clone now on bridle-adopt. Daemon up under launchd (dev.bridle.data-contracts) on 7407; its manager autostarted (idle, no tasks, no worker). Left for the human in the morning: `bridle token create orchestrator --project data-contracts` in a plain terminal; optional: add "data-contracts" to [mail] projects in ~/.bridle/config.toml. Then steps 3-4 (dc- tasks, vpaw first; one worker).

### note · external:orchestrator · 2026-10-10T02:23:32.195Z
orchestrator: token made by the human (via aide); waiter open on data-contracts. Step 3 done: dc-588s (vpaw, bug, high), dc-vueh (5hwh), dc-ne9h (y3cr), dc-7qe9 (8tz3), dc-p4ef (ptwp), dc-hwju (wdbz), dc-xrfv (wh6r), all pending. Step 4: I ready dc-588s once the machine-setup checks (br-88d4, br-751e, br-fpde) are through, per the PdM; the others stay pending for the human.
