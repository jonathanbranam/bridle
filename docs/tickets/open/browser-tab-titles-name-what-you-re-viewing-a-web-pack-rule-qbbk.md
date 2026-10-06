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
tasks: [br-qbbk, ui-qbbk]
---

## The ask

The human, verbatim (2026-10-06 ~4:20 PM ET, to the bridle-ui aide):

> If you can file a ticket still - I want nice titles in the browser; AGAIN, this should be a RULE added to the web dev pack. The browser title should be updated when the user has selected something in a route to show the name of what they are viewing. I have 10 tabs that all say "bridle" - I can't find my document or task or system tab.

The ask:
- **A rule in the web pack** (`workflow/packs/web/rules/`, beside web.input-clear-button.md and the others): the page title (`document.title`) names what the user is viewing, most specific first, and updates on every route change and selection, e.g. "<document or ticket name> - Document - bridle", "<task id> <task title> - Task - bridle", "System - bridle". A tab must be identifiable from its title alone. The human's words in the rule's "why".
- **bridle-ui follows it** on every page: the Tasks list and one task, Document (the open document's name or ticket title), Ticket, Specs (the open spec), System, Time, and the to-do list; it updates when the selection changes and falls back to the page name while loading. Tests check the title per route.
- "AGAIN": like the 16px inputs (g49c) and the clear button, the human wants a UI lesson written down as a pack rule so every web project gets it, not fixed once in bridle-ui.

## The human's follow-up: what goes in the title, and the favicon

The human, verbatim (2026-10-06 ~7:20 PM ET, to the bridle-ui aide):

> title bar should include lots of info, but we need to plan it carefully to fit on a crowded toolbar things to include:
>
> machine
> project
> tab
> filename
>
> that won't all fit; can we indicate machine and/or bridle with favicon? that would help
>
> Prefer the "name of the open thing" where possible, maybe an icon / emoji to tell if it's a todo, task, ticket, or spec?

What this adds:
- A crowded tab bar shows only the first ~10-15 characters of a title, so **the open thing's name goes first**, before anything else.
- **A type marker** in front of the name (an emoji or icon) says what kind of thing it is: to-do, task, ticket, spec, document. It replaces a "- Task -" word, which costs width.
- **The favicon carries what the title can't**: that it's bridle, and which machine. Project goes in the title after the name (it's short), or is left out where the name already says it (task IDs carry the project prefix).
- Tab (page) name only when nothing is selected ("System", "Tasks").

The aide's proposal, for the worker's brief (the human decides):
- Title: `<marker> <name> · <project>`, e.g. `📄 human-web-ui.md · bridle`, `🎫 Browser tab titles… · bridle`, `✅ ui-qbbk qbbk: browser tab… `; with nothing selected, the page name: `Tasks · bridle`.
- Favicon: the bridle mark, tinted with a colour per machine (from the machine name; the gateway already reports `machine`), so tabs from different machines differ at a glance. Set at runtime as an SVG data URL; no per-machine assets.
- The web pack rule says the general lesson: name first, type marker, favicon for app and host.
