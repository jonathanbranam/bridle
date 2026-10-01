---
id: cdez
title: "Quiet hours aren't quiet: agents still talk too long"
opened: 2026-10-01
repos: [bridle]
changes: []
specs: []
needs: []
see: [cvaq]
---

## The ask


The human, 2026-10-01, verbatim (via the advisor, during the `weekday-sleep` quiet period):

> Last thing - both you and orch are much too chatty during quiet time. I'd say the setting is a
> bust for tonight. Both are talking to me too long. So file a ticket that quiet needs to be
> quieter!! More forceful. We might need to inject something in a message received hook to push
> for stronger quiet behavior.
>
> I haven't tested the locked mode yet. Perhaps later.

## What's there now (at 0c741f6)

- `bridle focus gate` (`UserPromptSubmit` hook, `crates/bridle/src/focus.rs`) adds this to the
  prompt's context: "Quiet hours (<name>) until <time> ET. Lead your answer with a one-line nudge
  for the human to go back to what they should be doing, then keep the answer minimal."
- The advisor and orchestrator role text says the same (`workflow/base/roles/advisor.md`, Style;
  `workflow/base/roles/orchestrator.md`, "Quiet hours").
- In this session the advisor got the gate's text and still sent multi-paragraph answers,
  filed tickets and ran several tool calls per prompt.
