---
id: br-bdwh
title: No command to change which stages of a version 2 ticket need the human's review
kind: feature
created: 2026-10-18
created_by: external:advisor/tj7q
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ewk8, br-2ae8, br-tj7q]
---

## The ask

The human, 2026-10-18, to advisor tj7q, on filing this: "Yes approve".

There's no command to change which stages of a version 2 ticket need the human's signature after `ticket new --review`. `bridle ticket set` covers title, kind, theme, parent, repos, changes, specs, blocked_by and related only. During the human's try (br-ewk8) they asked to review every stage of br-2ae8 ("I want review each step of the test ticket"), and advisor tj7q had to edit the `stages:` block by hand (commit c3bd2bfc), which the designer role says never to do for frontmatter.

## Done when

- One command adds or removes `review: human` on a stage of an existing version 2 ticket (for example `bridle ticket set <id> review design,specs,tasks`, reusing `set` with the same comma list as `ticket new --review`; one name per action).
- It refuses a stage the ticket doesn't list, and changing the review of a stage already signed is refused or says what happens.
- docs/design/stages.md and docs/design/cli.md name it; a test covers adding and removing.
