---
id: br-tj7q
title: "Change stages on every ticket: attachments, proposal/design/specs/tasks files, per-ticket stage reviews, specs become bridle specs"
kind: epic
created: 2026-10-17
created_by: external:advisor/product-manager
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
theme: product-process
---

## The ask

The human, 2026-10-16 ~10:25 PM ET, to advisor (product-manager), verbatim:

> We need this epic done whatever it is called or the ladder:
>
> - attachments
> - change spec files on every ticket: proposal, design, specs, tasks
> - i can ask for review of specific stages per ticket or not. No other system yet. We can list
>   the stages per ticket that human wants to review.
> - agents wrote the files for all stages that are unblocked
> - specs become bridle specs when ticket is implemented

## What this means (advisor's reading; the designer confirms or corrects)

1. **Attachments.** A ticket can carry files besides its body (the attachments epic's "several
   documents on one ticket"; the human's MIME-parts idea). Images later.
2. **Four stage files on every ticket:** proposal, design, specs, tasks (the OpenSpec change
   shape). Each is a file attached to the ticket.
3. **Review per stage, chosen per ticket.** The ticket lists the stages the human wants to review
   (for example `review: [design, specs]`, or none). A listed stage waits for the human's
   approval before the next stage starts; an unlisted stage doesn't wait. No other review system
   in this epic (v2va's signed reviews stay separate).
4. **Agents write every unblocked stage.** A stage is unblocked when the stage before it is
   written and, if listed for review, approved. Agents (designer for proposal/design/specs,
   project manager for tasks) write each unblocked stage without being asked again.
5. **Specs become bridle specs on landing.** When the ticket is implemented, its specs stage
   becomes part of the project's bridle specs (`design/specs/`, docs/design/specs.md and
   spec-flow.md), so the specs outlive the ticket.

## Supersedes or absorbs

- The scheduler ladder (roadmap, 2026-10-09), steps 2-8: this epic is the pipeline the ladder
  was meant to prove. The rest of the scheduler (yfv5) becomes a ticket run through it.
- Epic `attachments` (roadmap epic 6): folded in as item 1.
- Gates epic (reviews-enforced): br-95mu (proposal and design approved before planning) is item
  3 for the stages the human lists; the v2va review slices stay separate.

## Done when

A new ticket gets the four stage files; the human can list the stages to review; agents write
each stage as it unblocks and wait only on the listed reviews; when the ticket lands its specs
are in the project's bridle specs. Proven on one real ticket end to end.

## First step

A designer pass: '## Design options' in this ticket (where stage files live: ticket folder,
state branch or attachments; how a stage is marked approved; how agents are told a stage
unblocked; how specs map into `design/specs/`), for the human's review. No build before that.
