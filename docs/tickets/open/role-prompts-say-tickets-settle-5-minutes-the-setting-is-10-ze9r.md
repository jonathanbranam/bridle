---
id: br-ze9r
title: Role prompts say tickets settle ~5 minutes; the setting is 10 minutes
kind: fix
created: 2026-10-18
created_by: external:advisor/tj7q
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-18, to advisor tj7q, verbatim: "yes it should be 10 min; 5 was the initial setting but it was still too fast". And earlier: "the 10 min hold exists and should be followed as normal".

The default `[tasks] settle` is 10m (docs/design/agent-host/roles-and-config.md), but two role prompts still say ~5 minutes:

- workflow/base/roles/project-manager.md:43, "Planned tickets still settle ~5 minutes after creation or a human edit"
- workflow/base/roles/manager.md:22, "A ticket settles ~5 minutes after creation or a human edit"

## Done when

- Both say 10 minutes and name the setting (`[tasks] settle`), so the prompts can't drift from the default again. List what restarts the settle period as the design doc does: creation, a human comment or edit, or a relay of the human's words.
- A grep of workflow/ and docs/ finds no other "5 minute" settle wording. (The "5m" in config.rs's parse error and test is an example duration, not the default; leave it.)
