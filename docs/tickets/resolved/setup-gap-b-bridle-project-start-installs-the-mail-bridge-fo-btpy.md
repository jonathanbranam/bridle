---
id: br-btpy
title: "Setup gap (b): bridle project start installs the mail bridge for the project"
kind: feature
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-dffz
resolved: 2026-10-16T10:02:58Z
---

## The ask

Part of epic br-dffz (the human, 2026-10-15 ~10:15 PM ET: fix as much as possible tonight; gaps a-e from br-rjd5 first). Seen in the hand-run of vesta-home setup (m-13894): `bridle project start` installed no mail bridge for the new project; add-a-machine step 5 (`bridle mail install`) is still manual, though afzu enrols every project in [mail] by default.

Wanted: `bridle project start` also installs and starts the project's mail bridge when the project is in `[mail] projects` and the machine has mail configured (idempotent: nothing to do if already installed), and says what it did or why it skipped. Update docs/context/add-a-machine.md and adding-a-project.md to drop the manual step.
