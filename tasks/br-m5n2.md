+++
id = "br-m5n2"
title = "Onboard data-contracts: start the trial that was prepared on 2026-09-28"
kind = "chore"
state = "open"
created_at = "2026-10-10T02:18:06.360Z"
updated_at = "2026-10-10T02:18:26.758624Z"
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
