---
id: br-ej8y
title: "Stages V1: stages you can see and steer (ticket page, review like a PR)"
kind: epic
created: 2026-10-17
created_by: external:advisor/tj7q
repos: [bridle, bridle-ui]
changes: []
specs: []
blocked_by: [br-tj7q]
related: [br-tj7q, ui-fwsx]
theme: product-process
---

## The ask

V1 of the change-stages work (the MVP is epic br-tj7q). Named and scoped by the human with advisor
tj7q, 2026-10-17 (thread of br-tj7q). Starts after the MVP lands.

## Outcome

The human sees and steers a ticket's stages in bridle-ui, reviews a stage like a PR, and writers get
told what was wrong.

## Scope

- The ticket page in bridle-ui: the stages as tabs (badge: waiting, written, needs your review,
  signed) and the other attachments in the document picker; task and ticket on one page (ui-fwsx;
  the human: "It's several clicks every time now").
- Review like a PR (the human, ~4:45 PM ET: "Think about a PR: it should basically work like a PR
  review. I can comment on different sections of the ticket, ask for changes, and edit it"):
  highlight comments on sections plus overall comments; Approve, or Request changes. Request
  changes sends the comments to a fresh writer of the same role, whose prompt is the stage's
  original rules and template, the file as written and the comments ("These are the rules that
  were used to write this ticket initially, and this is the ticket that was written. Now the human
  has added these comments. Please address them"); the result comes back for review.
- A check that a stage file has its template's headings (`ticket check`).
- A designer can send a stage back as blocked or needs input, with its reason on the thread, for a
  human review; the stage instructions say when (the human: "the proposal as written is not
  something we can implement, or the implementation is very complex").
- The interactive path: an advisor claims a stage and writes it live with the human through
  `bridle ticket next`, the same instructions and template as a background writer.

## Done when

The human reviews and approves, or requests changes on, a stage from the ticket page, and the
writer's redo comes back to them; the interactive path is used once.

## Not yet specified

The ticket page layout beyond tabs plus picker; how comments are anchored when the file changes.
