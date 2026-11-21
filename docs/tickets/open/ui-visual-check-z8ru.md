---
id: br-z8ru
title: "UI work ships without visual inspection: strict workflow and controls for UI tickets"
kind: re-evaluate
created: 2026-11-21
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

UI work reaches the human without anyone having looked at it, and reviewing it wastes their time. Bring the human OPTIONS for stricter workflow, guidelines and controls on UI tickets, across all projects. This is a design question: do not build one until the human picks.

The human, 2026-11-21 ~12:36 PM ET, via voice through aide@vesta, after a meta-notes-ui checkbox shipped with its check mark (a letter X) drawn outside the box (mu-9gqq), verbatim:

"Is there a meta-notes UI aid working, or can we file tickets there? I'm frankly kind of infuriated. The list check-off button thing doesn't look right. The visual style is terrible. When you click it off, the X isn't inside the box at all. Is it inside the circle at all? I don't really understand why, how that could be, how we have a component that obviously was never visually inspected. Also, isn't it just using React? Isn't there a really obvious and clear way to build a checkbox or something? I don't know. I could send a screenshot, but the X just isn't inside the box, and it's just a letter X. I don't know. It just looks ridiculous. Well, obviously, the whole point of bridle here is scaling, because there should be strict workflow, guidelines, and controls around how these things get built, and that's obviously not happening. We need to work on the bridle overall solution, especially for these UI-based things. They're never really doing what they should be doing. They aren't built in the right way, and it's a massive waste of my time to review them."

## What exists (pm-1)

Rule `verify-ui-in-a-browser` (`workflow/base/rules/verify-ui-in-a-browser.md`, must, roles manager and worker): the worker clicks through each acceptance item on a throwaway site and says what it saw; the manager repeats it before merging. It relies on agents' say-so: nothing is attached, and nothing checks that the thing looks right (a drawn X outside a box passes a "click and see it toggles" check).

## To research and bring as options (the human decides)

- Evidence attached: screenshots at phone and desktop widths, before and after, on the ticket; the manager must open them; a UI ticket without them cannot resolve.
- A looking step that judges appearance, not function: a reviewer agent (or the manager) that views the screenshots against the ticket and a short style guide.
- Guidelines: use the project's standard component library or native elements (a real checkbox) rather than hand-rolled ones; a per-project UI guideline file the workers read.
- Whether this is a new gate in bridle (resolve refused without attachments) or only rule/role text; and per-project opt-in (data-contracts and meta-notes differ).
- Edit existing rules rather than adding new ones (the human's 2026-11-17 rule), so the first candidate is a tightening of `verify-ui-in-a-browser`.

## Output

A comment with 2 to 3 options, each with cost and a recommendation, then a question to the human. No build. Also check mu-9gqq on meta-notes-ui for how it passed review (what the worker and manager reported).

## Model

Sonnet (research and design). Out of scope: fixing the checkbox itself (that is a ticket on meta-notes-ui; the aide said it is filing one).
