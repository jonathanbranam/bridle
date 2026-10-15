---
id: br-bft2
title: "Mail digest: local-time timestamps, grouped by day/week/older, and drop bare 'message sent' items"
kind: feature
created: 2026-10-15
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

The human, 2026-10-15 ~7:00 AM ET (to the aide, about the 06:30 digest), verbatim:

> - I'm going to need an enhancement to the UI so I can go through my inbox more quickly. I don't know. Maybe I can look through it and scroll, but I'm thinking of an easier UI. Don't design anything yet. I need to think about it.
> - I need timestamps on things. I'd like to know them in my local time, probably something like created and last updated if those are distinct, or if there could be updates for things.
> - I would want that stuff grouped into maybe reasonable things, like yesterday, last week, and older than that. That would help me find interesting things out of there, both things that I need to do and things that the system did, like tickets that were merged and everything. We just need some dates in there. It'd be helpful.
> - A bunch of these inbox things, I think, say "message sent," or the only text in there is "message sent" or something. I don't know what those are, but unless they have a link to something else, I don't feel like they're very useful.

Scope:
1. Timestamps in the human's local time (US Eastern) on each digest item: created, and last updated where that differs.
2. Group the items: yesterday, last week, older. Covers both the human's to-dos and what the system did (tickets merged and the like).
3. Find the items whose only text is "message sent" (or similar). Drop them unless they link to something.

Held: the first bullet (an easier UI for going through the inbox). The human: "Don't design anything yet. I need to think about it." No design or build for that part until the human picks it up.

Design: docs/design/mail.md (the digest).
