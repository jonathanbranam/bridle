---
id: qbbk
title: "Browser tab titles name what you're viewing: a web pack rule, and bridle-ui follows it on every page"
kind: feature
opened: 2026-10-06
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [g49c, m2pz]
tasks: []
---

## The ask

The human, verbatim (2026-10-06 ~4:20 PM ET, to the bridle-ui aide):

> If you can file a ticket still - I want nice titles in the browser; AGAIN, this should be a RULE added to the web dev pack. The browser title should be updated when the user has selected something in a route to show the name of what they are viewing. I have 10 tabs that all say "bridle" - I can't find my document or task or system tab.

The ask:
- **A rule in the web pack** (`workflow/packs/web/rules/`, beside web.input-clear-button.md and the others): the page title (`document.title`) names what the user is viewing, most specific first, and updates on every route change and selection, e.g. "<document or ticket name> - Document - bridle", "<task id> <task title> - Task - bridle", "System - bridle". A tab must be identifiable from its title alone. The human's words in the rule's "why".
- **bridle-ui follows it** on every page: the Tasks list and one task, Document (the open document's name or ticket title), Ticket, Specs (the open spec), System, Time, and the to-do list; it updates when the selection changes and falls back to the page name while loading. Tests check the title per route.
- "AGAIN": like the 16px inputs (g49c) and the clear button, the human wants a UI lesson written down as a pack rule so every web project gets it, not fixed once in bridle-ui.
