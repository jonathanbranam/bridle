---
id: br-8zts
title: Drop the task alias and old-name readers (22ab step 10), once all three machines are current and migrated
kind: chore
created: 2026-10-11
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-22ab
theme: tickets-and-release
---

## The ask

22ab step 10: remove the hidden `task` command alias, the old wire names and /v1/tasks routes, and
the old-name readers (the `tasks/` state-branch folder, `closed:` and the other old frontmatter
names), and do a final pass over the docs (docs/README.md, cli.md, storage.md, agent-host/).
Purging `resolved/` stays deferred.

## Gate (the human)

The human, 2026-10-11 ~6:35 PM ET (to advisor product-manager), verbatim: "Delay the removal of task
until confirmed that all three machine are on the latest version before that and have completed all
migrations". So this is not readied until the human confirms dalek, the NUC and the Windows PC run
the latest release and have applied every migration. Also after bridle-ui's ui-utp5 (UI says
ticket, including its to-do views) has landed and been installed, since the UI still calls the old
routes until then.
