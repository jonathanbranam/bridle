---
id: br-upxq
title: "Stages: a designer's stage must be on main before the human is asked to review it"
kind: bug
created: 2026-10-18
created_by: external:advisor/tj7q
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-ewk8, br-2ae8, br-ewhy, br-tj7q]
---

## The ask

Found in the human's first version 2 try (br-ewk8, ticket br-2ae8). The human, 2026-10-18, to advisor tj7q, verbatim:

> OK this is a failure. that is not how this can work. I'm not checking a branch to review a proposal. It has to be on main for me to review it.

## What happened

- The daemon spawned `designer-2ae8-proposal` (stage watch, br-ewhy). It wrote the proposal, ran `ticket stage-done br-2ae8 proposal`, committed 9ee0bc1f on `bridle/designer-2ae8-proposal`, messaged manager-2 and ended.
- The advisor was told "stage proposal written, waiting for the human's signature". But main, and so the web UI and `ticket show`, still had the empty template. manager-2 sat idle with the report; nothing landed the branch.
- docs/design/stages.md already says a stage counts as done for the daemon only once it's on the project checkout, "so the branch must land first", but nothing makes that happen, and the "waiting for signature" note goes out before it.
- Smaller: the designer's spawn prompt says "tell your manager with `bridle send`" without a name; `bridle send manager` failed and the designer had to look up manager-2 itself.

## Done when

- A stage a designer writes is on main before anyone is asked to review or sign it. Every stage, not only the ones with `review: human`. The design picks how: for example, stage writes commit straight to main the way ticket files do, or the stage watch lands the designer's branch itself as soon as `stage-done` is committed.
- The "waiting for the human's signature" note (and any message to an advisor or the human) goes out only once the stage is on main, and names the main commit.
- The designer's prompt names its manager (or the land step needs none).
- A test: a designer stage with `review: human` is readable on main (`ticket show`) when the signature request is sent.
- br-2ae8's proposal (9ee0bc1f) is on main so the try can continue (done by hand now; this ticket is for the next one).
