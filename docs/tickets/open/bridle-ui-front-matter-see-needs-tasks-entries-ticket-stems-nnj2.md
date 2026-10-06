---
id: nnj2
title: "bridle-ui: front matter see/needs/tasks entries (ticket stems and IDs) aren't links"
kind: bug
opened: 2026-10-06
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [fx7x, a3yd]
tasks: []
---

## The ask

The human, verbatim (2026-10-05 ~8 PM ET, to the bridle-ui aide): "When I open that ticket in the browser, it says, \"See these other tickets\" in the front matter, but those tickets are not linked."

Example: fx7x (docs/tickets/open/the-orchestrator-stays-running-fx7x.md) has `see: [the-humans-to-do-list-and-restart-checklist-ex9q, where-the-single-orchestrator-lives-hj4g]`; both tickets exist, neither is a link.

Cause (aide, from the code): the front-matter table renders each value with <Md>, but src/doc/links.ts only finds wiki links, docs/ paths, task/spec IDs and bare 4-char ticket IDs not preceded by "-". A ticket stem like `...-ex9q` is never a link candidate, so nothing is sent to links/resolve. The gateway already resolves stems (open/ then resolved/) and bare IDs.

- In the front matter, each item of a list field that names tickets or tasks (see, needs, tasks, and similar) is a link to that ticket or task.
- Also link IDs in task titles (Tasks list and task page render titles as plain text, so "(fx7x)" in a title is not clickable).
- A test with a stem, a bare ID and a task ID in front matter.
