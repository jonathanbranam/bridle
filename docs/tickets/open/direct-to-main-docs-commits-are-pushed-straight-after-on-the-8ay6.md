---
id: 8ay6
title: Direct-to-main docs commits are pushed straight after, on the owner's clone
kind: feature
opened: 2026-10-09
filed_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
needs: []
see: [j7r4, 8z7j, 8umh]
tasks: []
---

## The ask

From postmortem j7r4 (incident br-2y3m), recommendation 3: direct-to-main docs commits (tickets,
notes, made by the orchestrator, advisors and aides) are never pushed; only landings push, so
origin lags for hours. The human, 2026-10-09 14:04 EDT, verbatim (comment c3 on j7r4), on "Either
push after each direct-to-main docs commit": "Yes, I think this is a good pattern."

The ask: on the owner's integration clone, every direct commit on the integration branch is
pushed straight after (a bridle wrapper, a post-commit hook, or the daemon pushing on a short
timer when the branch is ahead). A rejected push is handled as 8umh says. Then operating-model.md's "the remote never lags the clone" is true for docs too.
Non-owner clones don't push (8z7j).
