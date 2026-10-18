---
id: br-57kc
title: "Orchestrator: drop the bridle token check step from every start"
kind: chore
created: 2026-10-18
created_by: external:aide
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Remove the startup step at workflow/base/roles/orchestrator.md:43-46 ("Then run `bridle token check` ..."), added by br-8rdn (60c9f9fa). Keep the `bridle token check` command itself, for the human to run when a project seems quiet or after a re-pair. Don't point any agent role at it: agents can't run it (see the human's second message below). Check the other places that tell the orchestrator to run it at start (docs/design/cli.md mentions the command) and drop those too, if there are any.

The human, 2026-10-18 ~7:00 PM ET (to aide), verbatim, on an orchestrator's explanation of the step:

> Is this accurate? That is a complete waste of tokens and time for a one-time problem. We need to remove that.

and right after:

> also - agents can't run that check anyway

Aide checked: accurate. Every orchestrator runs it at every start, because they all share the base role text.
