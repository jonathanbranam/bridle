---
id: gztq
title: "Evaluate the UI: can the human see which projects and which agents consume tokens?"
kind: research
opened: 2026-10-05
repos: [bridle, bridle-ui]
changes: []
specs: []
needs: []
see: [xypj]
tasks: []
---

## The ask

The human, 2026-10-05 ~4:10 PM ET, verbatim (to the aide, by voice): "Yeah, also, I need someone to evaluate the user interface. I'd really like to, I really want to be sure I can see which projects, which agents are consuming tokens."

The ask: someone evaluates the bridle UI (bridle-ui with the gateway) against this need: can the human see, across all projects, which projects and which agents are consuming tokens (and how much, over what period)? Report what the UI shows today, what's missing, and file the tickets to close the gaps.

What's there today: `bridle usage --json` has per-agent tokens and cost for one project's daemon; the account's budget is shared by every project (xypj).
