---
id: br-dy6a
title: "Product and project planning: one owner decides, sequences and plans each project's work"
kind: epic
created: 2026-10-18
created_by: external:advisor/product-manager
theme: product-process
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-jxk4, br-6h65, br-g5dm, br-tj7q, br-ej8y, br-ckbt, br-p7jf]
---

## The ask

The human, 2026-10-18 ~8:45 AM ET, to advisor (product-manager), verbatim:

> The other important thing after the current stages epics is the product/project planning work.
> Primary issues I'm aware of:
>
> 1. The pm is supposed to plan, but this planning is rarely necessary on top of what the PdM has
>    decided
> 2. Projects without a pm as orch to plan which is a bad clash in responsibility for that role
> 3. The PdM should be sequencing all of the work, do we need a pm at all?
> 4. Can we share a pm role among project within or between products?
> 5. Small projects have unclear instructions on how to move work forwards which has broken
>    repeatedly due to messaging and listening failures

(Item 2 reads "projects without a pm ask the orch to plan": today the orchestrator is acting PM
on every project but bridle, workflow/base/roles/orchestrator.md.)

## The epic

- **Outcome:** every project, big or small, in a product or not, has one clear owner for deciding,
  sequencing and planning its work, and work moves forward without the orchestrator planning and
  without depending on an agent hearing a message.
- **Done when (PdM draft, for the human to confirm):** an approved design answers the five
  questions above (PM vs PdM: one role, two, or shared across projects); every project's work is
  readied, sequenced and planned by that owner, not the orchestrator; a small project moves a
  ticket from filed to landed with no hand-carried message.
- **Theme:** `product-process`. Ordered after the Stages epics (tj7q MVP, ej8y V1, ckbt V2).

## Proposal

- **Problem:** planning is split three ways (PdM decides and sequences, pm-1 plans on bridle,
  the orchestrator plans everywhere else). pm-1's plan rarely adds to the PdM's decision; the
  orchestrator shouldn't plan at all (jxk4); small projects stall when a message isn't heard.
- **Approach:** grill the human (skill bridle-grill) on the five questions, then a designer pass
  with options, then child tickets. Stages (tj7q) already moves design and tasks into the ticket,
  which may leave little for a PM to plan: the design should start from what Stages leaves.
- **Scope:** who plans and sequences, and how a small project moves work. The several-PdMs-per-
  product question stays in epic `products` (g5dm), linked.

## Child and related tickets

- jxk4: re-evaluate how tasks get planned and ordered (child; its grill is this epic's first step).
- 6h65: a product manager that relates every ticket to open work (child).
- Input from xfb3 (resolved 2026-10-18 into this epic): a base rule that before an agent makes
  something a prerequisite, or ranks work on the premise that X exists or works, it checks in the
  code that the normal flow invokes X and cites file:line; a design doc saying "built" is not
  evidence.
- Related: g5dm (products), w2hj (spin up and settle), tj7q / ej8y (Stages), p7jf (agents).
