+++
id = "br-8zts"
title = "Drop the task alias and old-name readers (22ab step 10), once all three machines are current and migrated"
kind = "chore"
state = "pending"
created_at = "2026-10-11T22:39:40.135Z"
updated_at = "2026-10-17T02:06:01.598776Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "8zts"
+++

docs/tickets/open/drop-the-task-alias-and-old-name-readers-22ab-step-10-once-a-8zts.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:01.598Z
The human, 2026-10-16 ~10:05 PM ET, to advisor (product-manager): "All three machines on 0.7.5". ui-utp5 is integrated. Migrations NOT yet complete: on dalek, 'bridle migrate --all --dry-run' stops at track-web (0001-0005 pending-manual; 0006 fails on the protected dev branch, the bug br-p4f2). Gate stays shut until p4f2 lands and migrations run clean on every machine.
